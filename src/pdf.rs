use crate::models::{Finding, Severity};
use genpdf::elements::{Break, Paragraph};
use genpdf::{fonts, Document, Element};
use std::fmt;

#[derive(Debug)]
pub enum PdfError {
    FontError(String),
    RenderError(String),
}

impl fmt::Display for PdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfError::FontError(msg) => write!(f, "Font error: {}", msg),
            PdfError::RenderError(msg) => write!(f, "Render error: {}", msg),
        }
    }
}

impl std::error::Error for PdfError {}

/// Generate a professional PDF report from scan findings
///
/// Returns PDF bytes ready to be attached to an email or downloaded
pub fn generate_report(
    target_url: &str,
    grade: &str,
    scan_date: &str,
    framework: Option<&str>,
    platform: Option<&str>,
    findings: &[Finding],
) -> Result<Vec<u8>, PdfError> {
    // Load Liberation fonts (fallback to system fonts if not available)
    let font_family = fonts::from_files("fonts", "LiberationSans", None)
        .map_err(|e| {
            PdfError::FontError(format!(
                "Failed to load Liberation fonts: {}. Ensure Liberation Sans fonts are installed in fonts/ directory",
                e
            ))
        })?;

    // Create document with font family
    let mut doc = Document::new(font_family);

    // Set document metadata
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(15);
    doc.set_page_decorator(decorator);
    doc.set_line_spacing(1.25);

    // Page 1: Title and Executive Summary
    doc.push(
        Paragraph::new("ShipSecure Security Audit Report")
            .styled(genpdf::style::Style::new().bold().with_font_size(24))
    );

    doc.push(
        Paragraph::new("Deep Audit Report")
            .styled(genpdf::style::Style::new().with_font_size(16))
    );

    doc.push(Break::new(1.0));

    // Horizontal line separator
    doc.push(Paragraph::new("_".repeat(80)));
    doc.push(Break::new(0.5));

    // Scan metadata
    doc.push(Paragraph::new(format!("Target URL: {}", target_url)));
    doc.push(Paragraph::new(format!("Scan Date: {}", scan_date)));
    doc.push(Paragraph::new(format!("Overall Grade: {}", grade)));

    if let Some(fw) = framework {
        doc.push(Paragraph::new(format!("Framework Detected: {}", fw)));
    }

    if let Some(plt) = platform {
        doc.push(Paragraph::new(format!("Platform Detected: {}", plt)));
    }

    doc.push(Break::new(1.0));

    // Executive Summary
    doc.push(
        Paragraph::new("Executive Summary")
            .styled(genpdf::style::Style::new().bold().with_font_size(14))
    );
    doc.push(Break::new(0.5));

    let summary_text = format!(
        "This comprehensive security audit identified {} total security findings across your application. \
        Our scanners analyzed TLS configuration, exposed files, client-side security, containerization practices, \
        and framework-specific vulnerabilities{}.",
        findings.len(),
        if framework.is_some() { " specific to your detected framework" } else { "" }
    );
    doc.push(Paragraph::new(summary_text));
    doc.push(Break::new(1.0));

    // Findings count by severity
    let critical_count = findings.iter().filter(|f| f.severity == Severity::Critical).count();
    let high_count = findings.iter().filter(|f| f.severity == Severity::High).count();
    let medium_count = findings.iter().filter(|f| f.severity == Severity::Medium).count();
    let low_count = findings.iter().filter(|f| f.severity == Severity::Low).count();

    doc.push(
        Paragraph::new("Findings by Severity")
            .styled(genpdf::style::Style::new().bold().with_font_size(12))
    );
    doc.push(Paragraph::new(format!("  Critical: {}", critical_count)));
    doc.push(Paragraph::new(format!("  High:     {}", high_count)));
    doc.push(Paragraph::new(format!("  Medium:   {}", medium_count)));
    doc.push(Paragraph::new(format!("  Low:      {}", low_count)));
    doc.push(Break::new(1.5));

    // Findings by Severity (detailed sections)
    let severity_order = [Severity::Critical, Severity::High, Severity::Medium, Severity::Low];

    for severity in &severity_order {
        let severity_findings: Vec<&Finding> = findings
            .iter()
            .filter(|f| &f.severity == severity)
            .collect();

        // Skip severity levels with no findings
        if severity_findings.is_empty() {
            continue;
        }

        // Section header
        let severity_name = match severity {
            Severity::Critical => "Critical Severity Findings",
            Severity::High => "High Severity Findings",
            Severity::Medium => "Medium Severity Findings",
            Severity::Low => "Low Severity Findings",
        };

        doc.push(
            Paragraph::new(severity_name)
                .styled(genpdf::style::Style::new().bold().with_font_size(16))
        );
        doc.push(Break::new(0.5));

        // Each finding in this severity
        for finding in severity_findings {
            let title_prefix = if finding.vibe_code {
                "[VIBE-CODE] "
            } else {
                ""
            };

            doc.push(
                Paragraph::new(format!("{}{}", title_prefix, finding.title))
                    .styled(genpdf::style::Style::new().bold().with_font_size(11))
            );

            doc.push(Paragraph::new(format!("Scanner: {}", finding.scanner_name)));
            doc.push(Break::new(0.3));

            doc.push(Paragraph::new(format!("Description: {}", finding.description)));
            doc.push(Break::new(0.3));

            doc.push(
                Paragraph::new("Remediation:")
                    .styled(genpdf::style::Style::new().bold())
            );
            doc.push(Paragraph::new(&finding.remediation));
            doc.push(Break::new(1.0));
        }

        doc.push(Break::new(1.0));
    }

    // Remediation Roadmap
    doc.push(
        Paragraph::new("Remediation Roadmap")
            .styled(genpdf::style::Style::new().bold().with_font_size(16))
    );
    doc.push(Break::new(0.5));

    let roadmap_text = if critical_count > 0 {
        "Address Critical severity findings immediately - these represent active security vulnerabilities. \
        Then proceed to High severity issues, which could become exploitable under certain conditions. \
        Medium and Low severity findings should be addressed as part of regular security maintenance."
    } else if high_count > 0 {
        "Start with High severity findings - these represent significant security concerns. \
        Then address Medium and Low severity issues as part of your ongoing security posture improvement."
    } else if medium_count > 0 {
        "Address Medium severity findings to improve your security posture. \
        Low severity findings represent best practices and should be addressed during regular maintenance cycles."
    } else {
        "Address the identified Low severity findings as part of security best practices and regular maintenance."
    };

    doc.push(Paragraph::new(roadmap_text));
    doc.push(Break::new(1.5));

    // Footer
    doc.push(Paragraph::new("_".repeat(80)));
    doc.push(Break::new(0.5));
    doc.push(
        Paragraph::new("Generated by ShipSecure - https://shipsecure.ai")
            .styled(genpdf::style::Style::new().with_font_size(9))
    );

    // Render to bytes
    let mut buffer = Vec::new();
    doc.render(&mut buffer)
        .map_err(|e| PdfError::RenderError(format!("Failed to render PDF: {}", e)))?;

    Ok(buffer)
}
