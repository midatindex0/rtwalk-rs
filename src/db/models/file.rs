use async_graphql::SimpleObject;
use diesel::{
    backend::Backend, deserialize, serialize, sql_types::VarChar, AsExpression, FromSqlRow,
};

#[derive(AsExpression, FromSqlRow, Debug, SimpleObject)]
#[diesel(sql_type = VarChar)]
pub struct File {
    id: String,
}

impl File {
    pub fn new(id: String) -> Self {
        Self { id }
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
        <String as deserialize::FromSql<VarChar, B>>::from_sql(bytes).map(|id| File::new(id))
    }
}
