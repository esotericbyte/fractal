use either::Either;
use matrix_sdk::api::r0::user_directory::search_users::User;
use matrix_sdk::identifiers::UserId;
use matrix_sdk::{api::r0::membership::joined_members::RoomMember, identifiers::MxcUri};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Member {
    pub uid: UserId,
    pub alias: Option<String>,
    pub avatar: Option<Either<MxcUri, PathBuf>>,
}

impl Member {
    pub fn get_alias(&self) -> String {
        if let Some(ref alias) = self.alias {
            if !alias.is_empty() {
                return alias.clone();
            }
        }
        self.uid.to_string()
    }
}

impl PartialEq for Member {
    fn eq(&self, other: &Member) -> bool {
        self.uid == other.uid
    }
}

impl From<User> for Member {
    fn from(user: User) -> Self {
        Self {
            uid: user.user_id,
            alias: user.display_name,
            avatar: user.avatar_url.map(Either::Left),
        }
    }
}

impl From<(UserId, RoomMember)> for Member {
    fn from((uid, roommember): (UserId, RoomMember)) -> Self {
        Self {
            uid,
            alias: roommember.display_name,
            avatar: roommember.avatar_url.map(Either::Left),
        }
    }
}

// hashmap userid -> Member
pub type MemberList = HashMap<UserId, Member>;
