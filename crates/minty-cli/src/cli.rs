use crate::{ConfigFile, Result};

use clap::{Parser, Subcommand};
use minty::{text, PostSort, Url, Uuid};
use std::path::PathBuf;

mod env {
    macro_rules! var {
        ($name:ident) => {
            pub const $name: &str = concat!("MINTY", '_', stringify!($name));
        };
    }

    var!(CONFIG);
    var!(HUMAN_READABLE);
    var!(JSON);
    var!(LIMIT);
    var!(SERVER);
    var!(TAGS);
    var!(USER);
}

#[derive(Debug, Parser)]
#[command(name = "minty", version, arg_required_else_help = true)]
/// A Minty client for the command line
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "FILE",
        env = env::CONFIG,
        global = true
    )]
    /// Config file in TOML format
    pub config: Option<PathBuf>,

    #[arg(
        long,
        value_name = "ALIAS",
        env = env::SERVER,
        global = true,
        default_value = "default"
    )]
    /// The configured server to use
    pub server: String,

    #[arg(long, value_name = "ALIAS", env = env::USER, global = true)]
    /// The configured user to act as
    pub user: Option<String>,

    #[arg(short = 'H', long, env = env::HUMAN_READABLE, global = true)]
    /// Print data in a human-readable format
    pub human_readable: bool,

    #[arg(short, long, env = env::JSON, global = true)]
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
            env = env::LIMIT,
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

    /// Get information about all objects in the repo
    Objects {
        #[command(subcommand)]
        command: Objects,
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

    /// Display the given tags
    Tags {
        #[arg(value_delimiter = ' ', env = env::TAGS)]
        /// Tag IDs
        tags: Vec<Uuid>,
    },

    /// View a user's publicly available information
    User {
        /// User ID
        id: Uuid,
    },

    /// View or modify the logged in user
    Me {
        #[command(subcommand)]
        command: Option<Me>,
    },

    /// Log into a user account
    Login {
        #[arg(long, value_name = "ALIAS", env = env::USER)]
        /// The configured user to log in as
        user: String,
    },

    /// Close the current session
    Logout,

    /// Create a new account
    Signup {
        #[arg(long, value_name = "ALIAS", env = env::USER)]
        /// The configured user to sign up as
        user: String,

        /// New user's display name
        username: text::Name,

        /// Existing user's invitation token
        invitation: Option<String>,
    },

    /// Generate an invitation
    ///
    /// People without an account can use your invitation to sign up to
    /// this repo.
    Invite,

    /// Change your email address
    Email {
        /// The new email address
        email: text::Email,
    },

    /// Change your password
    Password,

    /// Grant privileges to another user
    Grant {
        #[command(subcommand)]
        command: Grant,
    },

    /// Revoke privileges from another user
    Revoke {
        #[command(subcommand)]
        command: Revoke,
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

        #[arg(short = 'u', long, value_name = "ID", conflicts_with = "drafts")]
        /// ID of the user who authored the post
        poster: Option<Uuid>,

        #[arg(short, long, value_name = "SORT", default_value = "created")]
        /// Result sorting
        sort_by: PostSort,

        #[arg(short, long, value_delimiter = ' ', env = env::TAGS)]
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

    /// Search for users
    User {
        /// Name or alias of the user to search for
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

        #[arg(
            short,
            long,
            value_name = "ID",
            value_delimiter = ' ',
            env = env::TAGS
        )]
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
        name: text::Name,
    },
}

#[derive(Debug, Subcommand)]
pub enum Me {
    /// Set your display name
    Rename {
        /// User's new display name
        name: text::Name,
    },

    /// Add an alias
    Aka {
        /// User's new alias
        alias: text::Name,
    },

    /// Set your description
    Desc {
        /// User's description
        description: Option<text::Description>,
    },

    /// Add a link to your user profile
    Ln {
        /// User's new link
        url: Url,
    },

    /// Delete the user account or metadata
    Rm {
        #[arg(short, long)]
        /// Do not prompt for confirmation before removal
        ///
        /// This is the default behavior if STDIN is not a terminal
        force: bool,

        #[command(subcommand)]
        command: Option<MeRm>,
    },
}

#[derive(Debug, Subcommand)]
pub enum MeRm {
    /// Remove an alias
    Alias {
        /// User alias to delete
        alias: Option<String>,
    },

    /// Remove a link
    Link {
        /// User links to delete
        sources: Vec<String>,
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
pub enum Objects {
    /// Print all object preview errors
    Errors,
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
        name: text::Name,
    },

    /// Add a tag alias
    Aka {
        /// Tag's new alias
        alias: text::Name,
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

#[derive(Debug, Subcommand)]
pub enum Grant {
    /// Grant another user administrator privileges
    Admin {
        /// The user's ID
        id: Uuid,
    },
}

#[derive(Debug, Subcommand)]
pub enum Revoke {
    /// Revoke another user's administrator privileges
    Admin {
        /// The user's ID
        id: Uuid,
    },
}

impl Cli {
    pub fn config(&self) -> Result<ConfigFile> {
        ConfigFile::read(self.config.clone())
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
