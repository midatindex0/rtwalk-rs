use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use tantivy;
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexReader, IndexWriter};

#[allow(missing_debug_implementations)]
pub struct IndexOp(Mutex<IndexWriter>, IndexReader, Index);

#[derive(Debug)]
pub struct SearchResults {
    inner: HashMap<i32, f32>,
}

impl SearchResults {
    pub fn ids(&self) -> Vec<i32> {
        self.inner.keys().copied().collect()
    }

    pub fn map_id_score(&self, id: i32) -> Option<f32> {
        self.inner.get(&id).copied()
    }
}

pub trait ToDoc {
    fn to_doc(self, schema: &Schema) -> anyhow::Result<Document>;
}

impl IndexOp {
    pub fn new(index: Index) -> Self {
        Self(
            Mutex::new(index.writer(50_000_000).unwrap()),
            index.reader().unwrap(),
            index,
        )
    }

    pub fn search(
        &self,
        query: &str,
        offset: usize,
        limit: usize,
    ) -> anyhow::Result<SearchResults> {
        let searcher = self.1.searcher();
        let mut default_fields = vec![];
        let schema = self.2.schema();
        let id = schema.get_field("id")?;
        for (field, _) in schema.fields() {
            default_fields.push(field);
        }
        default_fields.retain(|x| *x != id);
        let parser = QueryParser::for_index(&self.2, default_fields);
        let query = parser.parse_query(query)?;
        let results = searcher.search(&query, &TopDocs::with_limit(limit).and_offset(offset))?;
        let mut scored_id = HashMap::new();
        for (score, add) in results {
            let document = searcher.doc(add)?;
            scored_id.insert(
                document.get_first(id).unwrap().to_owned().as_i64().unwrap() as i32,
                score,
            );
        }
        Ok(SearchResults { inner: scored_id })
    }

    pub fn add<T: ToDoc>(&self, document: T) -> anyhow::Result<()> {
        let mut writer = self.0.lock().unwrap();
        let schema = self.2.schema();
        let to_insert = document.to_doc(&schema)?;
        writer.add_document(to_insert)?;
        writer.commit()?;
        Ok(())
    }
}

#[allow(missing_debug_implementations)]
pub struct SearchIndexInner {
    pub user: IndexOp,
    pub forum: IndexOp,
    pub post: IndexOp,
}

#[allow(missing_debug_implementations)]
pub struct SearchIndex(Arc<SearchIndexInner>);

impl Default for SearchIndex {
    fn default() -> Self {
        Self(Arc::new(SearchIndexInner {
            user: IndexOp::new(user_index()),
            forum: IndexOp::new(forum_index()),
            post: IndexOp::new(post_index()),
        }))
    }
}

impl Clone for SearchIndex {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Deref for SearchIndex {
    type Target = SearchIndexInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn user_index() -> Index {
    let mut builder = Schema::builder();
    builder.add_i64_field("id", INDEXED | STORED);
    builder.add_text_field("username", TEXT | COERCE | FAST);
    builder.add_text_field("display_name", TEXT | COERCE);
    builder.add_text_field("bio", TEXT | COERCE);

    let schema = builder.build();
    Index::open_or_create(
        MmapDirectory::open("./data/index/user")
            .expect("index directory does not exist at data/index/user"),
        schema,
    )
    .expect("Failed to open Index at data/index/user")
}

pub fn forum_index() -> Index {
    let mut builder = Schema::builder();
    builder.add_i64_field("id", INDEXED | STORED);
    builder.add_text_field("name", TEXT | COERCE | FAST);
    builder.add_text_field("display_name", TEXT | COERCE);
    builder.add_text_field("description", TEXT | COERCE);

    let schema = builder.build();
    Index::open_or_create(
        MmapDirectory::open("./data/index/forum")
            .expect("index directory does not exist at data/index/forum"),
        schema,
    )
    .expect("Failed to open Index at data/index/forum")
}

pub fn post_index() -> Index {
    let mut builder = Schema::builder();
    builder.add_i64_field("id", INDEXED | STORED);
    builder.add_text_field("tags", TEXT | COERCE);
    builder.add_text_field("title", TEXT | COERCE);
    builder.add_text_field("content", TEXT | COERCE);

    let schema = builder.build();
    Index::open_or_create(
        MmapDirectory::open("./data/index/post")
            .expect("index directory does not exist at data/index/post"),
        schema,
    )
    .expect("Failed to open Index at data/index/post")
}
