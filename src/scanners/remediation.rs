//! Framework-specific remediation generation for vibe-code vulnerabilities.
//!
//! Transforms generic security findings into copy-paste code fixes tailored to the
//! detected framework. Each remediation includes a targeted diff and 1-2 sentence
//! explanation of why the fix works.

use regex::Regex;

/// Internal vulnerability classification based on template ID and title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VulnType {
    /// Environment variable leaks (NEXT_PUBLIC_ secrets, .env exposure)
    EnvLeak,
    /// Supabase RLS disabled on sensitive tables
    SupabaseRls,
    /// Permissive Firebase rules
    FirebaseRules,
    /// API routes without authentication
    UnprotectedRoute,
    /// Netlify function/config exposure
    NetlifyExposure,
    /// Vercel deployment info leak
    VercelEnvLeak,
    /// Unknown vulnerability type
    Generic,
}

/// Classify vulnerability type based on Nuclei template ID or title keywords.
fn classify_vuln(template_id: &str, title: &str) -> VulnType {
    // First try exact template ID matches
    match template_id {
        "supabase-rls" => VulnType::SupabaseRls,
        "firebase-rules" => VulnType::FirebaseRules,
        "nextjs-env-leak" | "env-in-build-output" => VulnType::EnvLeak,
        "unprotected-api-routes" => VulnType::UnprotectedRoute,
        "netlify-function-exposure" => VulnType::NetlifyExposure,
        "vercel-env-leak" => VulnType::VercelEnvLeak,
        _ => {
            // Fallback to title keyword matching
            let title_lower = title.to_lowercase();
            if title_lower.contains("supabase") && title_lower.contains("rls") {
                VulnType::SupabaseRls
            } else if title_lower.contains("firebase") && title_lower.contains("rules") {
                VulnType::FirebaseRules
            } else if title_lower.contains("env") || title_lower.contains("environment variable") {
                VulnType::EnvLeak
            } else if title_lower.contains("unprotected") || title_lower.contains("api route") {
                VulnType::UnprotectedRoute
            } else if title_lower.contains("netlify") {
                VulnType::NetlifyExposure
            } else if title_lower.contains("vercel") {
                VulnType::VercelEnvLeak
            } else {
                VulnType::Generic
            }
        }
    }
}

/// Extract variable name from evidence containing NEXT_PUBLIC_ or PUBLIC_ prefixes.
///
/// Returns the variable name without the prefix, or None if extraction fails.
fn extract_var_name(raw_evidence: Option<&str>) -> Option<String> {
    let evidence = raw_evidence?;

    // Try to extract NEXT_PUBLIC_* variable name
    if let Some(caps) = Regex::new(r"NEXT_PUBLIC_([A-Z_][A-Z0-9_]*)")
        .ok()?
        .captures(evidence)
    {
        return Some(caps.get(1)?.as_str().to_string());
    }

    // Try to extract PUBLIC_* variable name (SvelteKit)
    if let Some(caps) = Regex::new(r"PUBLIC_([A-Z_][A-Z0-9_]*)")
        .ok()?
        .captures(evidence)
    {
        return Some(caps.get(1)?.as_str().to_string());
    }

    None
}

/// Extract table name from evidence for Supabase RLS remediation.
fn extract_table_name(raw_evidence: Option<&str>) -> Option<String> {
    let evidence = raw_evidence?;

    // Try to extract table name from common patterns
    if let Some(caps) = Regex::new(r#"table[:\s]+['"]?([a-z_][a-z0-9_]*)['"]?"#)
        .ok()?
        .captures(evidence)
    {
        return Some(caps.get(1)?.as_str().to_string());
    }

    None
}

/// Generate framework-specific remediation for a security finding.
///
/// # Arguments
/// * `template_id` - Nuclei template ID from the finding
/// * `title` - Finding title for classification fallback
/// * `framework` - Detected framework (lowercase: "nextjs", "vite_react", "sveltekit", "nuxt")
/// * `raw_evidence` - Optional evidence string from Nuclei output for context extraction
///
/// # Returns
/// Markdown-formatted remediation with code block and explanation.
pub fn generate_remediation(
    template_id: &str,
    title: &str,
    framework: Option<&str>,
    raw_evidence: Option<&str>,
) -> String {
    let vuln_type = classify_vuln(template_id, title);

    match (vuln_type, framework) {
        // EnvLeak + Next.js
        (VulnType::EnvLeak, Some("nextjs")) => {
            let var_name = extract_var_name(raw_evidence)
                .unwrap_or_else(|| "VAR_NAME".to_string());

            format!(
                r#"**In your `.env.local` file:**
```diff
- NEXT_PUBLIC_{var_name}=...
+ {var_name}=...  # Remove NEXT_PUBLIC_ prefix
```

**In your server-side code (`app/api/route.ts` or `pages/api/*.ts`):**
```typescript
const secret = process.env.{var_name};
```

Variables prefixed with `NEXT_PUBLIC_` are embedded in client-side JavaScript bundles and visible to anyone. Secrets must only exist server-side. **Rotate the exposed secret immediately.**"#
            )
        }

        // EnvLeak + SvelteKit
        (VulnType::EnvLeak, Some("sveltekit")) => {
            let var_name = extract_var_name(raw_evidence)
                .unwrap_or_else(|| "VAR_NAME".to_string());

            format!(
                r#"**In your `.env` file:**
```diff
- PUBLIC_{var_name}=...
+ {var_name}=...  # Remove PUBLIC_ prefix
```

**In your server-side code (`+page.server.ts` or `+server.ts`):**
```typescript
import {{ env }} from '$env/static/private';
const secret = env.{var_name};
```

SvelteKit exposes variables prefixed with `PUBLIC_` to the browser. Remove the prefix and access via `$env/static/private` for server-only values."#
            )
        }

        // EnvLeak + Nuxt
        (VulnType::EnvLeak, Some("nuxt")) => {
            format!(
                r#"**In your `nuxt.config.ts`:**
```diff
export default defineNuxtConfig({{
  runtimeConfig: {{
-   public: {{
-     secretKey: process.env.SECRET_KEY
-   }}
+   secretKey: process.env.SECRET_KEY  // Server-only (not under public)
  }}
}})
```

In Nuxt, values under `runtimeConfig.public` are sent to the client. Move secrets to the top level of `runtimeConfig` for server-only access."#
            )
        }

        // EnvLeak + generic (no framework or vite_react)
        (VulnType::EnvLeak, _) => {
            r#"**Remove the secret from client-side code.**

Environment variables containing secrets should never be included in browser-accessible JavaScript. Check your build configuration to ensure secrets are only available server-side.

Common patterns:
- **Next.js:** Remove `NEXT_PUBLIC_` prefix from secret vars
- **SvelteKit:** Remove `PUBLIC_` prefix and use `$env/static/private`
- **Nuxt:** Move from `runtimeConfig.public` to `runtimeConfig` root"#.to_string()
        }

        // SupabaseRls (framework-agnostic)
        (VulnType::SupabaseRls, _) => {
            let table_name = extract_table_name(raw_evidence)
                .unwrap_or_else(|| "table_name".to_string());

            format!(
                r#"**In your Supabase Dashboard or via SQL:**
```sql
-- Enable RLS on the affected table
ALTER TABLE {table_name} ENABLE ROW LEVEL SECURITY;

-- Create a policy for authenticated access only
CREATE POLICY "Users can only access own data"
ON {table_name}
FOR ALL
USING (auth.uid() = user_id);
```

Row Level Security (RLS) prevents unauthenticated access to your database tables via the public API. Without RLS, anyone with your Supabase URL and anon key can read all data."#
            )
        }

        // FirebaseRules (framework-agnostic)
        (VulnType::FirebaseRules, _) => {
            r#"**In your Firebase Console > Realtime Database > Rules:**
```json
{
  "rules": {
    ".read": "auth != null",
    ".write": "auth != null",
    "public": {
      ".read": true,
      ".write": false
    }
  }
}
```

Replace your current permissive rules (`".read": true, ".write": true`) with authenticated-only access. Create specific paths for genuinely public data."#.to_string()
        }

        // UnprotectedRoute + Next.js
        (VulnType::UnprotectedRoute, Some("nextjs")) => {
            r#"**Create `middleware.ts` in your project root:**
```typescript
import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

export function middleware(request: NextRequest) {
  if (request.nextUrl.pathname.startsWith('/api/')) {
    const token = request.cookies.get('auth-token');
    if (!token) {
      return NextResponse.json(
        { error: 'Unauthorized' },
        { status: 401 }
      );
    }
  }
  return NextResponse.next();
}

export const config = {
  matcher: '/api/:path*'
};
```

This middleware checks for authentication before any API route executes. Without it, your API routes are accessible to anyone who knows the URL."#.to_string()
        }

        // UnprotectedRoute + SvelteKit
        (VulnType::UnprotectedRoute, Some("sveltekit")) => {
            r#"**In your `src/hooks.server.ts`:**
```typescript
import type { Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
  if (event.url.pathname.startsWith('/api/')) {
    const session = event.cookies.get('session');
    if (!session) {
      return new Response(
        JSON.stringify({ error: 'Unauthorized' }),
        { status: 401, headers: { 'Content-Type': 'application/json' } }
      );
    }
  }
  return resolve(event);
};
```

SvelteKit's server hooks run before every request. Use them to enforce authentication on API routes."#.to_string()
        }

        // UnprotectedRoute + generic
        (VulnType::UnprotectedRoute, _) => {
            r#"**Add authentication middleware to your API routes.**

Ensure all sensitive API endpoints check for a valid session or token before processing requests. Unauthenticated API routes are a common vulnerability in AI-generated code because code generators often skip auth middleware."#.to_string()
        }

        // NetlifyExposure (framework-agnostic)
        (VulnType::NetlifyExposure, _) => {
            r#"**In your `netlify.toml`:**
```toml
[[redirects]]
  from = "/.netlify/*"
  to = "/404"
  status = 404
  force = true
```

This prevents direct access to internal Netlify function endpoints and configuration paths. Functions should only be accessed through your application's defined routes."#.to_string()
        }

        // VercelEnvLeak (framework-agnostic)
        (VulnType::VercelEnvLeak, _) => {
            r#"**Review your environment variable configuration in Vercel Dashboard.**

Deployment metadata variables (VERCEL_ENV, VERCEL_URL) are low-risk info disclosure, but avoid exposing them in client bundles. Use server-side only access or remove NEXT_PUBLIC_ prefix if present."#.to_string()
        }

        // Generic fallback
        (VulnType::Generic, _) => {
            r#"**Review and fix the identified security issue.**

Consult framework-specific security best practices for your stack. Common vibe-code vulnerabilities include exposed environment variables, missing authentication, and permissive database rules."#.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_vuln_from_template_id() {
        assert_eq!(classify_vuln("supabase-rls", ""), VulnType::SupabaseRls);
        assert_eq!(classify_vuln("firebase-rules", ""), VulnType::FirebaseRules);
        assert_eq!(classify_vuln("nextjs-env-leak", ""), VulnType::EnvLeak);
        assert_eq!(classify_vuln("env-in-build-output", ""), VulnType::EnvLeak);
        assert_eq!(classify_vuln("unprotected-api-routes", ""), VulnType::UnprotectedRoute);
        assert_eq!(classify_vuln("netlify-function-exposure", ""), VulnType::NetlifyExposure);
        assert_eq!(classify_vuln("vercel-env-leak", ""), VulnType::VercelEnvLeak);
    }

    #[test]
    fn test_classify_vuln_from_title() {
        assert_eq!(
            classify_vuln("unknown", "Supabase RLS Disabled"),
            VulnType::SupabaseRls
        );
        assert_eq!(
            classify_vuln("unknown", "Firebase permissive rules detected"),
            VulnType::FirebaseRules
        );
        assert_eq!(
            classify_vuln("unknown", "Environment variable leaked"),
            VulnType::EnvLeak
        );
        assert_eq!(
            classify_vuln("unknown", "Unprotected API route found"),
            VulnType::UnprotectedRoute
        );
        assert_eq!(
            classify_vuln("unknown", "Some other vulnerability"),
            VulnType::Generic
        );
    }

    #[test]
    fn test_extract_var_name_nextjs() {
        assert_eq!(
            extract_var_name(Some("Found NEXT_PUBLIC_STRIPE_SECRET_KEY in bundle")),
            Some("STRIPE_SECRET_KEY".to_string())
        );
        assert_eq!(
            extract_var_name(Some("NEXT_PUBLIC_API_KEY exposed")),
            Some("API_KEY".to_string())
        );
    }

    #[test]
    fn test_extract_var_name_sveltekit() {
        assert_eq!(
            extract_var_name(Some("Found PUBLIC_DATABASE_URL in client")),
            Some("DATABASE_URL".to_string())
        );
    }

    #[test]
    fn test_extract_var_name_no_match() {
        assert_eq!(extract_var_name(Some("No variables here")), None);
        assert_eq!(extract_var_name(None), None);
    }

    #[test]
    fn test_extract_table_name() {
        assert_eq!(
            extract_table_name(Some("RLS disabled on table: users")),
            Some("users".to_string())
        );
        assert_eq!(
            extract_table_name(Some("table 'profiles' has no RLS")),
            Some("profiles".to_string())
        );
    }

    #[test]
    fn test_generate_remediation_supabase_rls() {
        let result = generate_remediation(
            "supabase-rls",
            "Supabase RLS Disabled",
            Some("nextjs"),
            Some("table: users"),
        );

        assert!(result.contains("```sql"));
        assert!(result.contains("ALTER TABLE users ENABLE ROW LEVEL SECURITY"));
        assert!(result.contains("CREATE POLICY"));
        assert!(result.contains("Row Level Security"));
    }

    #[test]
    fn test_generate_remediation_nextjs_env_leak() {
        let result = generate_remediation(
            "nextjs-env-leak",
            "Environment Variable Exposed",
            Some("nextjs"),
            Some("Found NEXT_PUBLIC_API_KEY"),
        );

        assert!(result.contains("```diff"));
        assert!(result.contains("- NEXT_PUBLIC_API_KEY"));
        assert!(result.contains("+ API_KEY"));
        assert!(result.contains("```typescript"));
        assert!(result.contains("process.env.API_KEY"));
        assert!(result.contains("Rotate the exposed secret immediately"));
    }

    #[test]
    fn test_generate_remediation_nextjs_env_leak_generic_var() {
        let result = generate_remediation(
            "nextjs-env-leak",
            "Environment Variable Exposed",
            Some("nextjs"),
            None,
        );

        assert!(result.contains("```diff"));
        assert!(result.contains("NEXT_PUBLIC_VAR_NAME"));
        assert!(result.contains("process.env.VAR_NAME"));
    }

    #[test]
    fn test_generate_remediation_sveltekit_env_leak() {
        let result = generate_remediation(
            "env-in-build-output",
            "Environment Variable Exposed",
            Some("sveltekit"),
            Some("Found PUBLIC_SECRET"),
        );

        assert!(result.contains("```diff"));
        assert!(result.contains("- PUBLIC_SECRET"));
        assert!(result.contains("+ SECRET"));
        assert!(result.contains("$env/static/private"));
    }

    #[test]
    fn test_generate_remediation_env_leak_generic() {
        let result = generate_remediation(
            "nextjs-env-leak",
            "Environment Variable Exposed",
            None,
            None,
        );

        assert!(result.contains("Remove the secret from client-side code"));
        assert!(result.contains("Next.js:"));
        assert!(result.contains("SvelteKit:"));
        assert!(result.contains("Nuxt:"));
    }

    #[test]
    fn test_generate_remediation_unprotected_route_sveltekit() {
        let result = generate_remediation(
            "unprotected-api-routes",
            "Unprotected API Route",
            Some("sveltekit"),
            None,
        );

        assert!(result.contains("```typescript"));
        assert!(result.contains("hooks.server.ts"));
        assert!(result.contains("Handle"));
        assert!(result.contains("event.cookies.get"));
    }

    #[test]
    fn test_generate_remediation_firebase_rules() {
        let result = generate_remediation(
            "firebase-rules",
            "Firebase Rules Too Permissive",
            Some("nextjs"),
            None,
        );

        assert!(result.contains("```json"));
        assert!(result.contains("auth != null"));
        assert!(result.contains("Firebase Console"));
    }

    #[test]
    fn test_generate_remediation_netlify_exposure() {
        let result = generate_remediation(
            "netlify-function-exposure",
            "Netlify Function Exposed",
            None,
            None,
        );

        assert!(result.contains("```toml"));
        assert!(result.contains("netlify.toml"));
        assert!(result.contains("/.netlify/*"));
    }

    #[test]
    fn test_all_remediations_have_code_blocks() {
        let test_cases = vec![
            ("supabase-rls", "title", Some("nextjs"), None),
            ("firebase-rules", "title", None, None),
            ("nextjs-env-leak", "title", Some("nextjs"), None),
            ("nextjs-env-leak", "title", Some("sveltekit"), None),
            ("nextjs-env-leak", "title", Some("nuxt"), None),
            ("unprotected-api-routes", "title", Some("nextjs"), None),
            ("unprotected-api-routes", "title", Some("sveltekit"), None),
            ("netlify-function-exposure", "title", None, None),
            ("vercel-env-leak", "title", None, None),
        ];

        for (template_id, title, framework, evidence) in test_cases {
            let result = generate_remediation(template_id, title, framework, evidence);
            assert!(
                result.contains("```"),
                "Remediation for {} should contain code block",
                template_id
            );
        }
    }

    #[test]
    fn test_no_verify_sections() {
        let test_cases = vec![
            ("supabase-rls", "title", Some("nextjs"), None),
            ("firebase-rules", "title", None, None),
            ("nextjs-env-leak", "title", Some("nextjs"), None),
            ("unprotected-api-routes", "title", Some("sveltekit"), None),
            ("netlify-function-exposure", "title", None, None),
        ];

        for (template_id, title, framework, evidence) in test_cases {
            let result = generate_remediation(template_id, title, framework, evidence);
            assert!(
                !result.to_lowercase().contains("verify"),
                "Remediation for {} should NOT contain 'verify' sections",
                template_id
            );
        }
    }
}
