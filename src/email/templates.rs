use crate::email::FindingsSummary;

/// Generate HTML email for scan completion
pub fn scan_complete_html(
    target_url: &str,
    grade: &str,
    summary: &FindingsSummary,
    results_url: &str,
    expires_at: &str,
) -> String {
    let grade_color = match grade {
        "A+" | "A" | "A-" => "#10b981", // green
        "B+" | "B" | "B-" => "#f59e0b", // yellow
        "C+" | "C" | "C-" => "#f97316", // orange
        _ => "#ef4444", // red (D, F)
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Scan Complete</title>
</head>
<body style="margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif; background-color: #f3f4f6;">
    <table role="presentation" style="width: 100%; border-collapse: collapse;">
        <tr>
            <td align="center" style="padding: 40px 0;">
                <table role="presentation" style="width: 600px; max-width: 100%; border-collapse: collapse; background-color: #ffffff; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);">
                    <!-- Header -->
                    <tr>
                        <td style="padding: 40px 40px 20px; text-align: center;">
                            <h1 style="margin: 0; font-size: 28px; font-weight: 700; color: #111827;">Your Security Scan is Complete</h1>
                        </td>
                    </tr>

                    <!-- Grade Display -->
                    <tr>
                        <td style="padding: 20px 40px; text-align: center;">
                            <div style="display: inline-block; background-color: {grade_color}; color: #ffffff; font-size: 72px; font-weight: 700; width: 120px; height: 120px; line-height: 120px; border-radius: 12px; box-shadow: 0 4px 12px rgba(0,0,0,0.2);">
                                {grade}
                            </div>
                        </td>
                    </tr>

                    <!-- Target URL -->
                    <tr>
                        <td style="padding: 10px 40px; text-align: center;">
                            <p style="margin: 0; font-size: 16px; color: #6b7280;">
                                Target: <strong style="color: #111827;">{target_url}</strong>
                            </p>
                        </td>
                    </tr>

                    <!-- Findings Summary -->
                    <tr>
                        <td style="padding: 30px 40px;">
                            <h2 style="margin: 0 0 16px; font-size: 20px; font-weight: 600; color: #111827;">Findings Summary</h2>
                            <table role="presentation" style="width: 100%; border-collapse: collapse; background-color: #f9fafb; border-radius: 8px; padding: 16px;">
                                <tr>
                                    <td style="padding: 8px 16px; font-size: 14px; color: #6b7280;">Critical:</td>
                                    <td style="padding: 8px 16px; font-size: 14px; font-weight: 600; color: #111827; text-align: right;">{critical}</td>
                                </tr>
                                <tr>
                                    <td style="padding: 8px 16px; font-size: 14px; color: #6b7280;">High:</td>
                                    <td style="padding: 8px 16px; font-size: 14px; font-weight: 600; color: #111827; text-align: right;">{high}</td>
                                </tr>
                                <tr>
                                    <td style="padding: 8px 16px; font-size: 14px; color: #6b7280;">Medium:</td>
                                    <td style="padding: 8px 16px; font-size: 14px; font-weight: 600; color: #111827; text-align: right;">{medium}</td>
                                </tr>
                                <tr>
                                    <td style="padding: 8px 16px; font-size: 14px; color: #6b7280;">Low:</td>
                                    <td style="padding: 8px 16px; font-size: 14px; font-weight: 600; color: #111827; text-align: right;">{low}</td>
                                </tr>
                                <tr style="border-top: 1px solid #e5e7eb;">
                                    <td style="padding: 8px 16px; font-size: 14px; font-weight: 600; color: #111827;">Total:</td>
                                    <td style="padding: 8px 16px; font-size: 14px; font-weight: 600; color: #111827; text-align: right;">{total}</td>
                                </tr>
                            </table>
                        </td>
                    </tr>

                    <!-- Primary CTA -->
                    <tr>
                        <td style="padding: 10px 40px 20px;">
                            <a href="{results_url}" style="display: block; background-color: #3b82f6; color: #ffffff; text-decoration: none; padding: 16px 32px; border-radius: 8px; font-size: 16px; font-weight: 600; text-align: center; box-shadow: 0 2px 4px rgba(59,130,246,0.3);">
                                View Full Results
                            </a>
                        </td>
                    </tr>

                    <!-- Re-scan CTA -->
                    <tr>
                        <td style="padding: 10px 40px 30px; text-align: center;">
                            <p style="margin: 0 0 8px; font-size: 14px; color: #6b7280;">
                                Fixed some issues? Scan again to see your new score.
                            </p>
                            <a href="https://trustedgeaudit.com" style="color: #3b82f6; text-decoration: none; font-weight: 500; font-size: 14px;">
                                Run Another Scan
                            </a>
                        </td>
                    </tr>

                    <!-- Expiry Notice -->
                    <tr>
                        <td style="padding: 20px 40px; background-color: #fef3c7; border-top: 1px solid #fbbf24; border-bottom: 1px solid #fbbf24;">
                            <p style="margin: 0; font-size: 13px; color: #92400e; text-align: center;">
                                This link expires on <strong>{expires_at}</strong>. Results available for 3 days.
                            </p>
                        </td>
                    </tr>

                    <!-- Footer -->
                    <tr>
                        <td style="padding: 30px 40px; text-align: center; border-top: 1px solid #e5e7eb;">
                            <p style="margin: 0; font-size: 13px; color: #9ca3af;">
                                Powered by <strong style="color: #6b7280;">TrustEdge Audit</strong>
                            </p>
                        </td>
                    </tr>
                </table>
            </td>
        </tr>
    </table>
</body>
</html>"#,
        grade_color = grade_color,
        grade = grade,
        target_url = target_url,
        critical = summary.critical,
        high = summary.high,
        medium = summary.medium,
        low = summary.low,
        total = summary.total,
        results_url = results_url,
        expires_at = expires_at,
    )
}
