use crate::Surface;
use std::error::Error;
use std::fs;

fn bytes_to_u32(bytes: &[u8]) -> Result<u32, &str> {
    match bytes.len() {
        3 => Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0xff])),
        4 => Ok(u32::from_le_bytes(bytes.try_into().unwrap())),

        _ => Err("len of `bytes` must be 3 or 4"),
    }
}

pub fn load_bmp(path: &str) -> Result<Surface, Box<dyn Error>> {
    //doesnt work on non rgba bmps, a fix should be pretty easy??
    //this could most definetely be improved, but it works rn


    let px_offset_index = 0x0A;
    let img_size_index = 0x12;

    let file = fs::read(path)?;
    let pixel_array_offset = bytes_to_u32(&file[px_offset_index..px_offset_index + 4])? as usize;
    let width = bytes_to_u32(&file[img_size_index..img_size_index + 4])? as usize;
    let height = bytes_to_u32(&file[img_size_index + 4..img_size_index + 8])? as usize;



    let mut pixel_buffer: Vec<u32> = vec![0; width * height];
    for (i, value) in file[pixel_array_offset..]
        .chunks_exact(4)
        .map(|bytes| bytes_to_u32(bytes).unwrap())
        .enumerate()
    {
        let x = i % width;
        let y = height - 1 - i / width;
        pixel_buffer[x + y * width] = value;
    }

    Ok(Surface {
        width: width as usize,
        height: height as usize,
        pixel_buffer,
    })
}