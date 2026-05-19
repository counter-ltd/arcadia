//! Reads a runtime WASM module's manifest from its `arcadia.manifest` custom section.
//!
//! Pure — no `wasmi`, no instantiation. The section is part of the wasm binary format and
//! readable straight from the file bytes, which is what makes the two-phase lifecycle
//! possible: the loader reads the manifest to build a disabled stub, and only instantiates
//! the module when the user opts in.

use serde::Deserialize;

/// Name of the custom section a module embeds its manifest JSON in.
pub const MANIFEST_SECTION: &str = "arcadia.manifest";

/// One command a module declares. Declared in the manifest (not discovered at runtime) so
/// the stub can list commands before the module is instantiated.
#[derive(Debug, Clone, Deserialize)]
pub struct WasmManifestCommand {
    pub verb: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub required_permissions: Vec<String>,
}

/// A runtime WASM module's manifest, parsed from its `arcadia.manifest` custom section.
#[derive(Debug, Clone, Deserialize)]
pub struct WasmManifest {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub glyph: String,
    #[serde(default)]
    pub accent: String,
    #[serde(default)]
    pub required_permissions: Vec<String>,
    #[serde(default)]
    pub supported_platforms: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub abi_version: u32,
    #[serde(default)]
    pub commands: Vec<WasmManifestCommand>,
}

/// Decode an unsigned LEB128 integer at `*pos`, advancing `*pos`. `None` on truncation.
fn read_leb128_u32(bytes: &[u8], pos: &mut usize) -> Option<u32> {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;
    loop {
        let byte = *bytes.get(*pos)?;
        *pos += 1;
        result |= ((byte & 0x7f) as u32).checked_shl(shift)?;
        if byte & 0x80 == 0 {
            return Some(result);
        }
        shift += 7;
        if shift >= 32 {
            return None;
        }
    }
}

/// Extract the raw payload of the `arcadia.manifest` custom section from a wasm binary.
/// `None` if the bytes are not a wasm module or the section is absent.
pub fn read_manifest_section(bytes: &[u8]) -> Option<Vec<u8>> {
    // Header: magic "\0asm" + version word.
    if bytes.len() < 8 || &bytes[0..4] != b"\0asm" {
        return None;
    }
    let mut pos = 8;
    while pos < bytes.len() {
        let section_id = bytes[pos];
        pos += 1;
        let section_len = read_leb128_u32(bytes, &mut pos)? as usize;
        let section_end = pos.checked_add(section_len)?;
        if section_end > bytes.len() {
            return None;
        }
        if section_id == 0 {
            // Custom section: name-length-prefixed name, then content.
            let mut p = pos;
            let name_len = read_leb128_u32(bytes, &mut p)? as usize;
            let name_end = p.checked_add(name_len)?;
            if name_end <= section_end {
                let name = &bytes[p..name_end];
                if name == MANIFEST_SECTION.as_bytes() {
                    return Some(bytes[name_end..section_end].to_vec());
                }
            }
        }
        pos = section_end;
    }
    None
}

/// Read + parse a module's manifest from its wasm bytes.
pub fn parse(bytes: &[u8]) -> Result<WasmManifest, String> {
    let payload = read_manifest_section(bytes)
        .ok_or_else(|| format!("no '{MANIFEST_SECTION}' custom section in module"))?;
    let manifest: WasmManifest = serde_json::from_slice(&payload)
        .map_err(|e| format!("invalid manifest JSON: {e}"))?;
    if manifest.name.trim().is_empty() {
        return Err("manifest 'name' is empty".to_string());
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal wasm binary: header + one `arcadia.manifest` custom section.
    fn wasm_with_manifest(json: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"\0asm");
        out.extend_from_slice(&1u32.to_le_bytes());
        // custom section: id 0, then leb128(len), then leb128(name_len)+name + content
        let name = MANIFEST_SECTION.as_bytes();
        let mut payload = Vec::new();
        payload.push(name.len() as u8); // name_len fits in one leb128 byte
        payload.extend_from_slice(name);
        payload.extend_from_slice(json);
        out.push(0); // custom section id
        // section_len — small enough for one leb128 byte in tests
        assert!(payload.len() < 128);
        out.push(payload.len() as u8);
        out.extend_from_slice(&payload);
        out
    }

    #[test]
    fn parses_a_valid_manifest() {
        let json = br#"{"name":"hello","abi_version":1,"commands":[{"verb":"greet"}]}"#;
        let wasm = wasm_with_manifest(json);
        let m = parse(&wasm).expect("manifest must parse");
        assert_eq!(m.name, "hello");
        assert_eq!(m.abi_version, 1);
        assert_eq!(m.commands.len(), 1);
        assert_eq!(m.commands[0].verb, "greet");
    }

    #[test]
    fn rejects_non_wasm() {
        assert!(read_manifest_section(b"not wasm").is_none());
        assert!(parse(b"not wasm").is_err());
    }

    #[test]
    fn rejects_missing_section() {
        let mut wasm = Vec::new();
        wasm.extend_from_slice(b"\0asm");
        wasm.extend_from_slice(&1u32.to_le_bytes());
        assert!(read_manifest_section(&wasm).is_none());
    }

    #[test]
    fn rejects_empty_name() {
        let wasm = wasm_with_manifest(br#"{"name":""}"#);
        assert!(parse(&wasm).is_err());
    }
}
