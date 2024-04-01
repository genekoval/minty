use crate::db;

use minty::CommentData;

pub fn build_tree(comments: Vec<db::Comment>) -> Vec<CommentData> {
    let mut results = Vec::with_capacity(comments.len());

    let levels: Vec<_> = comments.chunk_by(|a, b| a.level == b.level).collect();
    let mut roots = levels
        .first()
        .map(|level| vec![level.iter()])
        .unwrap_or_default();

    while let Some(root) = 'next: {
        while let Some(chunk) = roots.last_mut() {
            let root = chunk.next();

            if root.is_some() {
                break 'next root;
            } else {
                roots.pop();
            }
        }

        None
    } {
        results.push(root.clone().into());

        if let Some(level) = levels.get(roots.len()) {
            if let Some(children) = level
                .chunk_by(|a, b| a.parent_id == b.parent_id)
                .find(|chunk| {
                    chunk.first().and_then(|comment| comment.parent_id)
                        == Some(root.id)
                })
            {
                roots.push(children.iter());
            }
        }
    }

    results
}
