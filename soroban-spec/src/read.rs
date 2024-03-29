use std::io::Cursor;

use hcnet_xdr::curr as hcnet_xdr;
use hcnet_xdr::{Limited, Limits, ReadXdr, ScSpecEntry};
use wasmparser::{BinaryReaderError, Parser, Payload};

// TODO: Move these functions into hcnet_xdr.

#[derive(thiserror::Error, Debug)]
pub enum ParseSpecBase64Error {
    #[error("parsing contract spec base64")]
    ParseBase64(base64::DecodeError),
    #[error("parsing contract spec xdr")]
    ParseXdr(hcnet_xdr::Error),
}

pub fn parse_base64(spec: &[u8]) -> Result<Vec<ScSpecEntry>, ParseSpecBase64Error> {
    let decoded = base64::decode(spec).map_err(ParseSpecBase64Error::ParseBase64)?;
    parse_raw(&decoded).map_err(ParseSpecBase64Error::ParseXdr)
}

pub fn parse_raw(spec: &[u8]) -> Result<Vec<ScSpecEntry>, hcnet_xdr::Error> {
    let cursor = Cursor::new(spec);
    let entries = ScSpecEntry::read_xdr_iter(&mut Limited::new(
        cursor,
        Limits {
            depth: 500,
            len: 0x1000000,
        },
    ))
    .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}

#[derive(thiserror::Error, Debug)]
pub enum FromWasmError {
    #[error("reading wasm")]
    Read(BinaryReaderError),
    #[error("parsing contract spec")]
    Parse(hcnet_xdr::Error),
    #[error("contract spec not found")]
    NotFound,
}

pub fn raw_from_wasm(wasm: &[u8]) -> Result<Vec<u8>, FromWasmError> {
    for payload in Parser::new(0).parse_all(wasm) {
        let payload = payload.map_err(FromWasmError::Read)?;
        if let Payload::CustomSection(section) = payload {
            if section.name() == "contractspecv0" {
                return Ok(section.data().to_vec());
            }
        };
    }
    Err(FromWasmError::NotFound)
}

pub fn base64_from_wasm(wasm: &[u8]) -> Result<String, FromWasmError> {
    let raw = raw_from_wasm(wasm)?;
    Ok(base64::encode(raw))
}

pub fn from_wasm(wasm: &[u8]) -> Result<Vec<ScSpecEntry>, FromWasmError> {
    let spec = raw_from_wasm(wasm)?;
    parse_raw(&spec).map_err(FromWasmError::Parse)
}
