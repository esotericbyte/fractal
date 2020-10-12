use gtk::prelude::*;

use crate::backend::{directory, HandleError};

use crate::app::RUNTIME;
use crate::appop::AppOp;

use crate::widgets;

use super::RoomSearchPagination;
use crate::model::room::Room;
use matrix_sdk::directory::RoomNetwork;
use matrix_sdk::thirdparty::ProtocolInstance;

impl AppOp {
    pub fn init_protocols(&self) {
        let session_client =
            unwrap_or_unit_return!(self.login_data.as_ref().map(|ld| ld.session_client.clone()));
        RUNTIME.spawn(async move {
            match directory::protocols(session_client).await {
                Ok(protocols) => {
                    APPOP!(set_protocols, (protocols));
                }
                Err(err) => {
                    err.handle_error();
                }
            }
        });
    }

    pub fn set_protocols(&self, protocols: Vec<ProtocolInstance>) {
        let combo = self
            .ui
            .builder
            .get_object::<gtk::ListStore>("protocol_model")
            .expect("Can't find protocol_model in ui file.");
        combo.clear();

        for p in protocols {
            combo.insert_with_values(None, &[0, 1], &[&p.desc, &p.network_id]);
        }
    }

    pub fn search_rooms(&mut self) {
        let session_client =
            unwrap_or_unit_return!(self.login_data.as_ref().map(|ld| ld.session_client.clone()));
        let other_protocol_radio = self
            .ui
            .builder
            .get_object::<gtk::RadioButton>("other_protocol_radio")
            .expect("Can't find other_protocol_radio in ui file.");

        let protocol: Option<String> = if other_protocol_radio.get_active() {
            let protocol_combo = self
                .ui
                .builder
                .get_object::<gtk::ComboBox>("protocol_combo")
                .expect("Can't find protocol_combo in ui file.");

            let protocol_model = self
                .ui
                .builder
                .get_object::<gtk::ListStore>("protocol_model")
                .expect("Can't find protocol_model in ui file.");

            let active = protocol_combo.get_active().map_or(-1, |uint| uint as i32);

            protocol_model
                .iter_nth_child(None, active)
                .and_then(|it| protocol_model.get_value(&it, 1).get().ok()?)
        } else {
            None
        };

        let q = self
            .ui
            .builder
            .get_object::<gtk::Entry>("directory_search_entry")
            .expect("Can't find directory_search_entry in ui file.");

        let other_homeserver_radio = self
            .ui
            .builder
            .get_object::<gtk::RadioButton>("other_homeserver_radio")
            .expect("Can't find other_homeserver_radio in ui file.");

        let other_homeserver_url = self
            .ui
            .builder
            .get_object::<gtk::EntryBuffer>("other_homeserver_url")
            .expect("Can't find other_homeserver_url in ui file.");

        let homeserver = if other_homeserver_radio.get_active() {
            Some(other_homeserver_url.get_text())
        } else {
            None
        };

        if !self.directory_pagination.has_more() {
            let directory = self
                .ui
                .builder
                .get_object::<gtk::ListBox>("directory_room_list")
                .expect("Can't find directory_room_list in ui file.");
            for ch in directory.get_children() {
                directory.remove(&ch);
            }

            let directory_stack = self
                .ui
                .builder
                .get_object::<gtk::Stack>("directory_stack")
                .expect("Can't find directory_stack in ui file.");
            let directory_spinner = self
                .ui
                .builder
                .get_object::<gtk::Box>("directory_spinner")
                .expect("Can't find directory_spinner in ui file.");
            directory_stack.set_visible_child(&directory_spinner);

            self.directory.clear();

            q.set_sensitive(false);
        }

        let search_term = Some(q.get_text().to_string()).filter(|s| !s.is_empty());
        if let RoomSearchPagination::NoMorePages = self.directory_pagination {
            // there are no more rooms. We don't need to request for more
            return;
        }

        let rooms_since: Option<String> = self.directory_pagination.clone().into();
        RUNTIME.spawn(async move {
            let query = directory::room_search(
                session_client,
                homeserver.as_deref(),
                search_term.as_deref(),
                protocol
                    .as_deref()
                    .map_or(RoomNetwork::Matrix, RoomNetwork::ThirdParty),
                rooms_since.as_deref(),
            )
            .await;

            match query {
                Ok((rooms, rooms_since)) => {
                    APPOP!(append_directory_rooms, (rooms, rooms_since));
                }
                Err(err) => {
                    err.handle_error();
                }
            }
        });
    }

    #[inline]
    pub fn load_more_rooms(&mut self) {
        self.search_rooms();
    }

    pub fn append_directory_rooms(&mut self, rooms: Vec<Room>, rooms_since: Option<String>) {
        self.directory_pagination = rooms_since
            .map(RoomSearchPagination::Next)
            .unwrap_or(RoomSearchPagination::NoMorePages);

        let directory = self
            .ui
            .builder
            .get_object::<gtk::ListBox>("directory_room_list")
            .expect("Can't find directory_room_list in ui file.");
        directory.get_style_context().add_class("room-directory");

        let directory_stack = self
            .ui
            .builder
            .get_object::<gtk::Stack>("directory_stack")
            .expect("Can't find directory_stack in ui file.");
        let directory_clamp = self
            .ui
            .builder
            .get_object::<libhandy::Clamp>("directory_clamp")
            .expect("Can't find directory_clamp in ui file.");
        directory_stack.set_visible_child(&directory_clamp);

        let mut sorted_rooms = rooms;
        sorted_rooms.sort_by_key(|a| -i128::from(a.n_members));

        for r in sorted_rooms.iter() {
            self.directory.push(r.clone());
            let rb = widgets::RoomBox::new(&r, &self);
            let room_widget = rb.widget();
            directory.add(&room_widget);
        }

        let q = self
            .ui
            .builder
            .get_object::<gtk::Entry>("directory_search_entry")
            .expect("Can't find directory_search_entry in ui file.");
        q.set_sensitive(true);
    }

    pub fn reset_directory_state(&self) {
        let q = self
            .ui
            .builder
            .get_object::<gtk::Entry>("directory_search_entry")
            .expect("Can't find directory_search_entry in ui file.");
        q.set_sensitive(true);

        let directory_stack = self
            .ui
            .builder
            .get_object::<gtk::Stack>("directory_stack")
            .expect("Can't find directory_stack in ui file.");
        let directory_clamp = self
            .ui
            .builder
            .get_object::<libhandy::Clamp>("directory_clamp")
            .expect("Can't find directory_clamp in ui file.");
        directory_stack.set_visible_child(&directory_clamp);
    }
}
