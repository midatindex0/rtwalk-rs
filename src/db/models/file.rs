use async_graphql::{ComplexObject, SimpleObject};
use diesel::{
    backend::Backend, deserialize, serialize, sql_types::VarChar, AsExpression, FromSqlRow,
};
use opendal::Operator;
use serde::{Deserialize, Serialize};

use std::io::prelude::*;

use crate::constants::CDN_PATH;

#[derive(AsExpression, FromSqlRow, Debug, Clone, SimpleObject, Deserialize, Serialize)]
#[diesel(sql_type = VarChar)]
#[graphql(complex)]
pub struct File {
    id: String,
}

#[ComplexObject]
impl File {
    /// Returns path that includes the CDN prefix
    async fn absolute_path(&self) -> String {
        format!("{}/{}", CDN_PATH, self.id)
    }
}

impl File {
    pub fn new(id: String) -> Self {
        Self { id }
    }

    pub async fn save(&self, fp: &mut std::fs::File, op: &Operator) -> anyhow::Result<()> {
        let mut buffer = vec![];
        fp.read_to_end(&mut buffer)?;
        op.write(&self.id, buffer).await?;
        Ok(())
    }
}

impl<B: Backend> serialize::ToSql<VarChar, B> for File
where
    String: serialize::ToSql<VarChar, B>,
{
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, B>) -> serialize::Result {
        <String as serialize::ToSql<VarChar, B>>::to_sql(&self.id, out)
    }
}

impl<B: Backend> deserialize::FromSql<VarChar, B> for File
where
    String: deserialize::FromSql<VarChar, B>,
{
    fn from_sql(bytes: <B as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        <String as deserialize::FromSql<VarChar, B>>::from_sql(bytes).map(File::new)
    }
}
