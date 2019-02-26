extern crate byteorder;
extern crate pyo3;

use byteorder::{BigEndian, ReadBytesExt};
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3::wrap_pyfunction;
use pyo3::{IntoPy, Py};

#[pyfunction]
fn loads(py: Python, format: String, buf: &PyBytes, offset: usize) -> PyResult<Py<PyTuple>> {
    let mut bitcount = 0;
    let mut bits = 0;
    let mut current_offset = offset;
    let values = PyList::empty(py);
    let buf = buf.as_bytes();

    for p in format.chars() {
        match p {
            'b' => {
                if bitcount == 0 {
                    bits = buf.get(current_offset..current_offset + 1).unwrap()[0];
                }

                bitcount = 8;
                values.append((bits & 1) == 1)?;
                bits >>= 1;
                bitcount -= 1;
                current_offset += 1;
            }
            'o' => {
                bitcount = 0;
                bits = 0;
                values.append((&buf[current_offset..]).read_u8().unwrap())?;
                current_offset += 1;
            }
            'B' => {
                bitcount = 0;
                bits = 0;
                values.append((&buf[current_offset..]).read_u16::<BigEndian>().unwrap())?;
                current_offset += 2;
            }
            'l' => {
                bitcount = 0;
                bits = 0;
                values.append((&buf[current_offset..]).read_u32::<BigEndian>().unwrap())?;
                current_offset += 4;
            }
            'L' => {
                bitcount = 0;
                bits = 0;
                values.append((&buf[current_offset..]).read_u64::<BigEndian>().unwrap())?;
                current_offset += 8;
            }
            'f' => {
                bitcount = 0;
                bits = 0;
                values.append((&buf[current_offset..]).read_f32::<BigEndian>().unwrap())?;
                current_offset += 4;
            }
            's' => {
                // TODO: Handle Unicode
                bitcount = 0;
                bits = 0;
                let slen = (&buf[current_offset..]).read_u8().unwrap() as usize;
                current_offset += 1;
                if current_offset + slen > buf.len() {
                    values.append(std::str::from_utf8(&buf[current_offset..]).unwrap())?;
                } else {
                    values.append(
                        std::str::from_utf8(&buf[current_offset..current_offset + slen]).unwrap(),
                    )?;
                }
                current_offset += slen;
            }
            'S' => {
                // TODO: Handle Unicode
                bitcount = 0;
                bits = 0;
                let slen = (&buf[current_offset..]).read_u32::<BigEndian>().unwrap() as usize;
                current_offset += 4;
                if current_offset + slen > buf.len() {
                    values.append(std::str::from_utf8(&buf[current_offset..]).unwrap())?;
                } else {
                    values.append(
                        std::str::from_utf8(&buf[current_offset..current_offset + slen]).unwrap(),
                    )?;
                }
                current_offset += slen;
            }
            'x' => {
                let blen = (&buf[current_offset..]).read_u32::<BigEndian>().unwrap() as usize;
                current_offset += 4;
                if current_offset + blen > buf.len() {
                    values.append(PyBytes::new(py, &buf[current_offset..]))?;
                } else {
                    values.append(PyBytes::new(py, &buf[current_offset..current_offset + blen]))?;
                }
                current_offset += blen;
            }
            // TODO: Handle complex objects
            _ => {
                // TODO: Once the exception type moves to rust, rewrite this
                // Or find out how to use the import_exception! macro
                let pycode = &format!("from amqp.exceptions import FrameSyntaxError;raise FrameSyntaxError('Table type {} not handled by amqp.')", p);
                py.run(pycode, None, None)?;
            }
        }
    }
    Ok((values, current_offset).into_py(py))
}

#[pymodule]
fn amqp_serialization(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(loads))?;

    Ok(())
}
