use crate::io::{AsyncStreamExt, StreamDecode};
use crate::protocol::message::{BackendMessage, BackendMessageFormat};
use crate::protocol::text::ColumnDefinition;
use crate::protocol::ServerContext;
use crate::{XuguColumn, XuguTypeInfo};
use sqlx_core::ext::ustr::UStr;
use sqlx_core::{Error, HashMap};

#[derive(Debug)]
pub struct RowDescription {
    pub fields: Vec<ColumnDefinition>,
}

fn to_xugu_column(def: &ColumnDefinition, ordinal: usize) -> Result<XuguColumn, Error> {
    // if the alias is empty, use the alias
    // only then use the name
    let name = match (def.name()?, def.alias()?) {
        (_, alias) if !alias.is_empty() => UStr::new(alias),
        (name, _) => UStr::new(name),
    };

    let type_info = XuguTypeInfo::from_column(def);

    Ok(XuguColumn {
        name,
        type_info,
        ordinal,
        flags: Some(def.flags),
    })
}

impl RowDescription {
    pub fn convert_columns(self) -> Result<(Vec<XuguColumn>, HashMap<UStr, usize>), Error> {
        let num_columns = self.fields.len();
        let mut column_names = HashMap::with_capacity(num_columns * 2);
        let mut columns = Vec::with_capacity(num_columns);

        for ordinal in 0..num_columns {
            let def = &self.fields[ordinal];

            let column = to_xugu_column(def, ordinal)?;

            column_names.insert(column.name.clone(), ordinal);
            // 列名不区分大小写，将大写和小写列名都加入
            column_names.insert(column.name.to_uppercase().into(), ordinal);
            column_names.insert(column.name.to_lowercase().into(), ordinal);
            columns.push(column);
        }

        Ok((columns, column_names))
    }
}

impl BackendMessage for RowDescription {
    const FORMAT: BackendMessageFormat = BackendMessageFormat::RowDescription;

    async fn decode_body<S: AsyncStreamExt>(
        stream: &mut S,
        cnt: ServerContext,
    ) -> Result<Self, Error> {
        let num_columns = stream.read_i32().await?;
        let mut fields = Vec::with_capacity(num_columns as usize);

        for _ in 0..num_columns {
            let def = ColumnDefinition::decode_with(stream, cnt).await?;
            fields.push(def);
        }
        Ok(Self { fields })
    }
}
