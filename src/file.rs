use crate::drawing::val_from_rgba;
use crate::Surface;
use std::error::Error;
use std::fs::File;

pub fn load_png(path: &str) -> Result<Surface, Box<dyn Error>> {
    let decoder = png::Decoder::new(File::open(path)?);
    let mut reader = decoder.read_info()?;

    let mut buf = vec![0; reader.output_buffer_size()];

    let info = reader.next_frame(&mut buf).unwrap();

    let bytes = &buf[..info.buffer_size()];
    let width = info.width as usize;
    let height = info.height as usize;

    let buf: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|chunk| {
            let [r, g, b, a]: [_; 4] = chunk.try_into().unwrap();
            val_from_rgba(r, g, b, a)
        })
        .collect();
    assert_eq!(buf.len(), width * height);
    Ok(Surface {
        width,
        height,
        pixel_buffer: buf,
    })
}
