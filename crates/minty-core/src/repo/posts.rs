use super::Repo;

use crate::{db, Error, Result};

use minty::{
    PostParts, PostPreview, PostQuery, SearchResult, Uuid, Visibility,
};

pub struct Posts<'a> {
    repo: &'a Repo,
}

impl<'a> Posts<'a> {
    pub(super) fn new(repo: &'a Repo) -> Self {
        Self { repo }
    }

    pub async fn add(&self, user: Uuid, parts: &PostParts) -> Result<Uuid> {
        let mut tx = self.repo.database.begin().await?;

        let post = tx
            .create_post(
                user,
                parts.title.as_ref().map(|t| t.as_ref()).unwrap_or(""),
                parts.description.as_ref().map(|d| d.as_ref()).unwrap_or(""),
                parts.visibility.map(db::Visibility::from_minty),
                parts.objects.as_deref().unwrap_or(&[]),
                parts.posts.as_deref().unwrap_or(&[]),
                parts.tags.as_deref().unwrap_or(&[]),
            )
            .await?;

        self.repo.search.add_post(&post).await?;

        tx.commit().await?;
        Ok(post.id)
    }

    pub(super) async fn build(
        &self,
        posts: Vec<db::PostPreview>,
    ) -> Result<Vec<PostPreview>> {
        let objects = posts
            .iter()
            .filter_map(|post| post.preview.clone())
            .collect();

        let mut objects = self
            .repo
            .bucket
            .get_object_previews(objects)
            .await?
            .into_iter();

        let posts = posts
            .into_iter()
            .map(|post| PostPreview {
                id: post.id,
                poster: post.poster.map(Into::into),
                title: post.title,
                preview: if post.preview.is_some() {
                    objects.next()
                } else {
                    None
                },
                comment_count: post.comment_count,
                object_count: post.object_count,
                created: post.created,
            })
            .collect();

        Ok(posts)
    }

    pub async fn find(
        &self,
        user_id: Option<Uuid>,
        mut query: PostQuery,
    ) -> Result<SearchResult<PostPreview>> {
        if user_id.is_none() && query.visibility != Visibility::Public {
            return Err(Error::Unauthenticated(None));
        }

        if query.visibility == Visibility::Draft {
            query.poster = user_id;
        }

        let results = self.repo.search.find_posts(&query).await?;
        let posts = self.repo.database.read_posts(&results.hits).await?;
        let posts = self.build(posts).await?;

        Ok(SearchResult {
            total: results.total,
            hits: posts,
        })
    }
}
