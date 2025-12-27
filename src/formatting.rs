use std::collections::BTreeMap;

use genemichaels::FormatConfig;

pub fn format(file: syn::File) -> anyhow::Result<String> {
    Ok(
        genemichaels::format_ast(file, &FormatConfig::default(), BTreeMap::new())
            .map_err(|e| anyhow::format_err!("formatting source file: {:?}", e))?
            .rendered,
    )
}
