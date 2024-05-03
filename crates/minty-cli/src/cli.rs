use crate::{Config, Result};

use clap::{Parser, Subcommand};
use minty::{text, PostSort, Url, Uuid};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, arg_required_else_help = true)]
/// A Minty client for the command line
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "FILE",
        env = "MINTY_CONFIG",
        global = true
    )]
    /// Config file in TOML format
    pub config: Option<PathBuf>,

    #[arg(
        long,
        value_name = "ALIAS",
        env = "MINTY_SERVER",
        global = true,
        default_value = "default"
    )]
    /// The configured server to use
    pub server: String,

    #[arg(short = 'H', long, env = "MINTY_HUMAN_READABLE", global = true)]
    /// Print data in a human-readable format
    pub human_readable: bool,

    #[arg(short, long, env = "MINTY_JSON", global = true)]
    /// Print data in JSON format
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Get detailed information about the server
    About,

    /// Search for things
    Find {
        #[arg(
            short,
            long,
            value_name = "OFFSET",
            default_value = "0",
            global = true
        )]
        /// Starting entry offset
        from: u32,

        #[arg(
            short = 'n',
            long,
            value_name = "LIMIT",
            env = "MINTY_LIMIT",
            default_value = "50",
            global = true
        )]
        /// Maximum number of hits to return
        size: u32,

        #[command(subcommand)]
        command: Find,
    },

    /// Create new tags, posts, etc.
    New {
        #[command(subcommand)]
        command: New,
    },

    /// Read or modify a post
    Post {
        /// Post ID
        id: Uuid,

        #[command(subcommand)]
        command: Option<Post>,
    },

    /// Get information about an object
    Obj {
        /// Object ID
        id: Uuid,

        #[command(subcommand)]
        command: Option<Object>,
    },

    /// Read a single comment
    Comment {
        /// Comment ID
        id: Uuid,

        #[command(subcommand)]
        command: Option<Comment>,
    },

    /// Read a post's comments
    Comments {
        /// Post ID
        post: Uuid,
    },

    /// Reply to a comment
    Reply {
        /// Parent comment ID
        comment: Uuid,

        /// Comment text
        ///
        /// If not present, content will be read from STDIN.
        content: Option<text::Comment>,
    },

    /// Read about or modify a tag
    Tag {
        /// Tag ID
        id: Uuid,

        #[command(subcommand)]
        command: Option<Tag>,
    },

    /// Fetch the entire repo as a JSON object
    Export,
}

#[derive(Debug, Subcommand)]
pub enum Comment {
    /// Change a comment's content
    Edit {
        /// The comment's new comment
        ///
        /// If not present, content will be read from STDIN.
        content: Option<text::Comment>,
    },

    /// Delete a comment
    Rm {
        #[arg(short, long)]
        /// Do not prompt for confirmation before removal
        ///
        /// This is the default behavior if STDIN is not a terminal
        force: bool,

        #[arg(short, long)]
        /// Delete all child comments
        recursive: bool,
    },
}

#[derive(Debug, Subcommand)]
#[command(flatten_help = true)]
pub enum Find {
    /// Search for posts
    Post {
        #[arg(short, long)]
        /// Only search for post drafts
        drafts: bool,

        #[arg(short, long, value_name = "SORT", default_value = "created")]
        /// Result sorting
        sort_by: PostSort,

        #[arg(short, long)]
        /// Search for posts with the given tags
        tag: Vec<Uuid>,

        /// Title/description text to search for
        text: Option<String>,
    },

    /// Search for tags
    Tag {
        /// Name or alias of the tag to search for
        name: String,
    },
}

#[derive(Debug, Subcommand)]
#[command(flatten_help = true)]
pub enum New {
    /// Create a new post
    Post {
        #[arg(short = 'T', long, value_name = "TEXT")]
        /// Post title
        title: Option<text::PostTitle>,

        #[arg(short = 'D', long, value_name = "TEXT")]
        /// Post description
        description: Option<text::Description>,

        #[arg(short, long)]
        /// Do not publish the newly created post
        draft: bool,

        #[arg(short, long, value_name = "ID")]
        /// Link related posts
        post: Option<Vec<Uuid>>,

        #[arg(short, long, value_name = "ID")]
        /// Add tags to the post
        tag: Option<Vec<Uuid>>,

        /// Files to attach to the post
        objects: Vec<String>,
    },

    /// Comment on a post
    Comment {
        /// Post ID
        post: Uuid,

        /// Comment text
        ///
        /// If not present, content will be read from STDIN.
        content: Option<text::Comment>,
    },

    /// Create a new tag
    Tag {
        /// New tag's name
        name: text::TagName,
    },
}

#[derive(Debug, Subcommand)]
pub enum Object {
    /// Download an object's data
    Get {
        #[arg(short, long)]
        /// Do not overwrite an existing file
        no_clobber: bool,

        /// Write output to a file instead of stdout
        destination: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum Post {
    /// Set a post's title
    Title {
        /// The post's title text
        ///
        /// If not present, the title will be read from STDIN.
        text: Option<text::PostTitle>,
    },

    /// Set a post's description
    Desc {
        /// The post's description text
        ///
        /// If not present, the description will be read from STDIN.
        text: Option<text::Description>,
    },

    /// Attach additional files to a post
    Obj {
        #[arg(short, long, value_name = "ID")]
        /// Existing object to insert in front of
        ///
        /// If omitted, new objects will be appended to the end.
        destination: Option<Uuid>,

        #[arg(required = true)]
        /// Files to attach to the post
        ///
        /// This may be the ID of an object that already exists on this server,
        /// a path to a local file, or an HTTP(S) URL.
        objects: Vec<String>,
    },

    /// Link related posts to this post
    Ln {
        #[arg(required = true)]
        /// IDs of existing posts
        posts: Vec<Uuid>,
    },

    /// Add tags to a post
    Tag {
        #[arg(required = true)]
        /// IDs of tags to add
        tags: Vec<Uuid>,
    },

    /// Publish a draft and make it visible to others
    Publish,

    /// Delete a post
    Rm {
        #[arg(short, long)]
        /// Do not prompt for confirmation before removal
        ///
        /// This is the default behavior if STDIN is not a terminal
        force: bool,

        #[command(subcommand)]
        command: Option<PostRm>,
    },
}

#[derive(Debug, Subcommand)]
pub enum PostRm {
    /// Remove attached files from a post
    Obj {
        #[arg(required = true)]
        /// IDs of objects to remove
        objects: Vec<Uuid>,
    },

    /// Remove related posts from a post
    Related {
        #[arg(required = true)]
        /// IDs of related posts to remove
        posts: Vec<Uuid>,
    },

    /// Remove tags from a post
    Tag {
        #[arg(required = true)]
        /// IDs of tags to remove
        tags: Vec<Uuid>,
    },
}

#[derive(Debug, Subcommand)]
pub enum Tag {
    /// Set a tag's primary name
    Rename {
        /// Tag's new primary name
        name: text::TagName,
    },

    /// Add a tag alias
    Aka {
        /// Tag's new alias
        alias: text::TagName,
    },

    /// Set a tag's description
    Desc {
        /// Tag's description
        description: Option<text::Description>,
    },

    /// Add a link to a tag
    Ln {
        /// Tag's new link
        url: Url,
    },

    /// Delete a tag
    Rm {
        #[arg(short, long)]
        /// Do not prompt for confirmation before removal
        ///
        /// This is the default behavior if STDIN is not a terminal
        force: bool,

        #[command(subcommand)]
        command: Option<TagRm>,
    },
}

#[derive(Debug, Subcommand)]
pub enum TagRm {
    /// Remove a tag's alias
    Alias {
        /// Tag alias to delete
        alias: Option<String>,
    },

    /// Remove a tag's link
    Link {
        /// Tag links to delete
        sources: Vec<String>,
    },
}

impl Cli {
    pub fn config(&self) -> Result<Config> {
        Config::read(self.config.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
