//! PDF aus JPEG-Seiten: jedes Bild unverändert (DCTDecode) als eine Seite.

struct JpegInfo {
    width: u32,
    height: u32,
    components: u8,
    dpi: f64,
}

fn jpeg_info(data: &[u8]) -> Result<JpegInfo, String> {
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return Err("Keine JPEG-Datei.".into());
    }
    let mut dpi = 300.0;
    let mut i = 2;
    while i + 4 <= data.len() {
        if data[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = data[i + 1];
        let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
        if len < 2 {
            return Err("Beschädigte JPEG-Datei.".into());
        }
        let seg = &data[i + 4..(i + 2 + len).min(data.len())];
        if marker == 0xE0 && seg.starts_with(b"JFIF\0") && seg.len() >= 12 && seg[7] == 1 {
            let x = u16::from_be_bytes([seg[8], seg[9]]);
            if x > 0 {
                dpi = x as f64;
            }
        }
        if (0xC0..=0xCF).contains(&marker)
            && ![0xC4, 0xC8, 0xCC].contains(&marker)
            && seg.len() >= 6
        {
            return Ok(JpegInfo {
                height: u16::from_be_bytes([seg[1], seg[2]]) as u32,
                width: u16::from_be_bytes([seg[3], seg[4]]) as u32,
                components: seg[5],
                dpi,
            });
        }
        i += 2 + len;
    }
    Err("JPEG-Grösse nicht gefunden.".into())
}

/// Bild → JPEG (JPEGs bleiben, wie sie sind).
pub fn to_jpeg(data: &[u8]) -> Result<Vec<u8>, String> {
    if is_jpeg(data) {
        return Ok(data.to_vec());
    }
    let img = image::load_from_memory(data)
        .map_err(|e| e.to_string())?
        .to_rgb8();
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90)
        .encode_image(&img)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

pub fn is_jpeg(data: &[u8]) -> bool {
    jpeg_info(data).is_ok()
}

/// Bettet die JPEGs unverändert (DCTDecode) als je eine Seite in ein PDF ein.
pub fn make_pdf(pages: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    if pages.is_empty() {
        return Err("Keine Seiten gescannt.".into());
    }
    let mut pdf: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets: Vec<usize> = Vec::new();
    let page_obj = |n: usize| 3 + n * 3;
    let push = |pdf: &mut Vec<u8>, offsets: &mut Vec<usize>, body: &[u8]| {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n", offsets.len()).as_bytes());
        pdf.extend_from_slice(body);
        pdf.extend_from_slice(b"\nendobj\n");
    };

    push(&mut pdf, &mut offsets, b"<< /Type /Catalog /Pages 2 0 R >>");
    let kids: Vec<String> = (0..pages.len())
        .map(|n| format!("{} 0 R", page_obj(n)))
        .collect();
    push(
        &mut pdf,
        &mut offsets,
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            kids.join(" "),
            pages.len()
        )
        .as_bytes(),
    );

    for (n, data) in pages.iter().enumerate() {
        let info = jpeg_info(data)?;
        let w = info.width as f64 * 72.0 / info.dpi;
        let h = info.height as f64 * 72.0 / info.dpi;
        let (img, contents) = (page_obj(n) + 1, page_obj(n) + 2);
        push(
            &mut pdf,
            &mut offsets,
            format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w:.2} {h:.2}] /Resources << /XObject << /Im0 {img} 0 R >> >> /Contents {contents} 0 R >>").as_bytes(),
        );
        let cs = match info.components {
            1 => "/DeviceGray",
            4 => "/DeviceCMYK",
            _ => "/DeviceRGB",
        };
        let mut image = format!(
            "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace {cs} /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n",
            info.width,
            info.height,
            data.len()
        )
        .into_bytes();
        image.extend_from_slice(data);
        image.extend_from_slice(b"\nendstream");
        push(&mut pdf, &mut offsets, &image);
        let draw = format!("q {w:.2} 0 0 {h:.2} 0 0 cm /Im0 Do Q");
        push(
            &mut pdf,
            &mut offsets,
            format!("<< /Length {} >>\nstream\n{draw}\nendstream", draw.len()).as_bytes(),
        );
    }

    let xref = pdf.len();
    pdf.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1).as_bytes(),
    );
    for off in &offsets {
        pdf.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            offsets.len() + 1
        )
        .as_bytes(),
    );
    Ok(pdf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_from_jpeg() {
        let jpeg: Vec<u8> = vec![
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x01,
            0x00, 0x96, 0x00, 0x96, 0x00, 0x00, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00,
            0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xD9,
        ];
        let pdf = make_pdf(&[jpeg.clone(), jpeg]).unwrap();
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(String::from_utf8_lossy(&pdf).contains("/Count 2"));
    }
}
