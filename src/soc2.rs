use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use crate::models::AuditResult;

pub fn export_soc2_pdf(result: &AuditResult, path: &str) -> anyhow::Result<()> {

    let (doc, page1, layer1) = PdfDocument::new("SOC2 Compliance Report", Mm(210.0), Mm(297.0), "Layer 1");
    let layer = doc.get_page(page1).get_layer(layer1);

    let font = doc.add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| anyhow::anyhow!("Failed to add font: {}", e))?;

    layer.use_text(
        format!("Status: {}", result.status),
        14.0,
        Mm(10.0),
        Mm(280.0),
        &font,
    );

    layer.use_text(
        format!("Risk Score: {}", result.risk_score),
        12.0,
        Mm(10.0),
        Mm(270.0),
        &font,
    );

    let file = File::create(path).map_err(|e| anyhow::anyhow!("Failed to create file: {}", e))?;
    doc.save(&mut BufWriter::new(file))
        .map_err(|e| anyhow::anyhow!("Failed to save PDF: {}", e))?;
    Ok(())
}
