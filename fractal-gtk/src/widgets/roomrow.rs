extern crate pango;
extern crate url;
extern crate gtk;

use self::url::Url;
use self::gtk::prelude::*;

use fractal_api;
use fractal_api::util::AvatarMode;
use fractal_api::util::draw_identicon;

use types::Room;

use util::glib_thread_prelude::*;

use widgets;
use widgets::AvatarExt;


const ICON_SIZE: i32 = 20;


// Room row for the room sidebar. This widget shows the room avatar, the room name and the unread
// messages in the room
// +-----+--------------------------+------+
// | IMG | Fractal                  |  32  |
// +-----+--------------------------+------+
pub struct RoomRow {
    pub room: Room,
    pub icon: widgets::Avatar,
    pub text: gtk::Label,
    pub notifications: gtk::Label,
}

impl RoomRow {
    pub fn new(room: Room, baseu: &Url) -> RoomRow {
        let name = room.name.clone().unwrap_or_default();
        let avatar = room.avatar.clone().unwrap_or_default();
        let icon = widgets::Avatar::avatar_new(Some(ICON_SIZE));
        let text = gtk::Label::new(name.clone().as_str());
        text.set_alignment(0.0, 0.0);
        text.set_ellipsize(pango::EllipsizeMode::End);

        let notifications = gtk::Label::new(&format!("{}", room.notifications)[..]);
        if let Some(style) = notifications.get_style_context() {
            style.add_class("notify-badge");
        }

        icon.default(String::from("avatar-default-symbolic"), Some(ICON_SIZE));
        download_avatar(baseu, room.id.clone(), name, avatar, &icon);

        RoomRow {
            room,
            icon,
            text,
            notifications,
        }
    }

    pub fn set_notifications(&self, n: i32) {
        self.notifications.set_text(&format!("{}", n));
        match n {
            0 => self.notifications.hide(),
            _ => self.notifications.show(),
        }
    }

    pub fn widget(&self) -> gtk::Box {
        let b = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        if let Some(style) = b.get_style_context() {
            style.add_class("room-row");
        }

        b.pack_start(&self.icon, false, false, 5);
        b.pack_start(&self.text, true, true, 0);
        b.pack_start(&self.notifications, false, false, 5);
        b.show_all();

        if self.room.notifications == 0 {
            self.notifications.hide();
        }

        b
    }
}

fn download_avatar(baseu: &Url,
                   rid: String,
                   name: String,
                   avatar: String,
                   image: &widgets::Avatar) {

    let url = baseu.clone();
    let img = image.clone();
    glib_thread!(Result<String, Error>,
        || {
            match avatar {
                ref s if s.is_empty() => identicon!(&rid, name),
                _ => fractal_api::util::dw_media(&url, &avatar, true, None, 40, 40),
            }
        },
        |rc: Result<String, Error>| {
            if let Ok(c) = rc {
                img.circle(c, Some(ICON_SIZE));
            }
        }
    );
}
