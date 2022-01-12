use adw::subclass::prelude::*;
use gtk::{glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate};
use log::warn;

use super::Emoji;
use crate::components::SpinnerButton;
use crate::contrib::screenshot;
use crate::contrib::QRCode;
use crate::contrib::QRCodeExt;
use crate::contrib::QrCodeScanner;
use crate::session::user::UserExt;
use crate::session::verification::{
    IdentityVerification, SasData, VerificationMode, VerificationState,
};
use crate::spawn;
use gettextrs::gettext;
use matrix_sdk::encryption::verification::QrVerificationData;

mod imp {
    use super::*;
    use glib::subclass::InitializingObject;
    use glib::SignalHandlerId;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/FractalNext/identity-verification-widget.ui")]
    pub struct IdentityVerificationWidget {
        pub request: RefCell<Option<IdentityVerification>>,
        #[template_child]
        pub qrcode: TemplateChild<QRCode>,
        #[template_child]
        pub emoji_row_1: TemplateChild<gtk::Box>,
        #[template_child]
        pub emoji_row_2: TemplateChild<gtk::Box>,
        #[template_child]
        pub emoji_match_btn: TemplateChild<SpinnerButton>,
        #[template_child]
        pub emoji_not_match_btn: TemplateChild<SpinnerButton>,
        #[template_child]
        pub start_emoji_btn: TemplateChild<SpinnerButton>,
        #[template_child]
        pub start_emoji_btn2: TemplateChild<SpinnerButton>,
        #[template_child]
        pub start_emoji_btn3: TemplateChild<SpinnerButton>,
        #[template_child]
        pub scan_qr_code_btn: TemplateChild<SpinnerButton>,
        #[template_child]
        pub accept_btn: TemplateChild<SpinnerButton>,
        #[template_child]
        pub decline_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub take_screenshot_btn2: TemplateChild<SpinnerButton>,
        #[template_child]
        pub take_screenshot_btn3: TemplateChild<SpinnerButton>,
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub qr_code_scanner: TemplateChild<QrCodeScanner>,
        #[template_child]
        pub done_btn: TemplateChild<gtk::Button>,
        pub state_handler: RefCell<Option<SignalHandlerId>>,
        pub name_handler: RefCell<Option<SignalHandlerId>>,
        #[template_child]
        pub label1: TemplateChild<gtk::Label>,
        #[template_child]
        pub label2: TemplateChild<gtk::Label>,
        #[template_child]
        pub label3: TemplateChild<gtk::Label>,
        #[template_child]
        pub label4: TemplateChild<gtk::Label>,
        #[template_child]
        pub label5: TemplateChild<gtk::Label>,
        #[template_child]
        pub label6: TemplateChild<gtk::Label>,
        #[template_child]
        pub label7: TemplateChild<gtk::Label>,
        #[template_child]
        pub label8: TemplateChild<gtk::Label>,
        #[template_child]
        pub label9: TemplateChild<gtk::Label>,
        #[template_child]
        pub label10: TemplateChild<gtk::Label>,
        #[template_child]
        pub label11: TemplateChild<gtk::Label>,
        #[template_child]
        pub label12: TemplateChild<gtk::Label>,
        #[template_child]
        pub label13: TemplateChild<gtk::Label>,
        #[template_child]
        pub label14: TemplateChild<gtk::Label>,
        #[template_child]
        pub label15: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IdentityVerificationWidget {
        const NAME: &'static str = "IdentityVerificationWidget";
        type Type = super::IdentityVerificationWidget;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            SpinnerButton::static_type();
            QRCode::static_type();
            Emoji::static_type();
            QrCodeScanner::static_type();

            klass.install_action("verification.decline", None, move |obj, _, _| {
                obj.decline();
            });

            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for IdentityVerificationWidget {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "request",
                    "Request",
                    "The Object holding the data for the verification",
                    IdentityVerification::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "request" => obj.set_request(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "request" => obj.request().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.accept_btn
                .connect_clicked(clone!(@weak obj => move |button| {
                    let priv_ = imp::IdentityVerificationWidget::from_instance(&obj);
                    button.set_loading(true);
                    priv_.decline_btn.set_sensitive(false);
                    obj.accept();
                }));

            self.emoji_match_btn
                .connect_clicked(clone!(@weak obj => move |button| {
                    let priv_ = imp::IdentityVerificationWidget::from_instance(&obj);
                    button.set_loading(true);
                    priv_.emoji_not_match_btn.set_sensitive(false);
                    if let Some(request) = obj.request() {
                        request.emoji_match();
                    }
                }));

            self.emoji_not_match_btn
                .connect_clicked(clone!(@weak obj => move |button| {
                    let priv_ = imp::IdentityVerificationWidget::from_instance(&obj);
                    button.set_loading(true);
                    priv_.emoji_match_btn.set_sensitive(false);
                    if let Some(request) = obj.request() {
                        request.emoji_not_match();
                    }
                }));

            self.start_emoji_btn
                .connect_clicked(clone!(@weak obj => move |button| {
                    let priv_ = imp::IdentityVerificationWidget::from_instance(&obj);
                    button.set_loading(true);
                    priv_.scan_qr_code_btn.set_sensitive(false);
                    if let Some(request) = obj.request() {
                        request.start_sas();
                    }
                }));
            self.start_emoji_btn2
                .connect_clicked(clone!(@weak obj => move |button| {
                    let priv_ = imp::IdentityVerificationWidget::from_instance(&obj);
                    button.set_loading(true);
                    priv_.take_screenshot_btn2.set_sensitive(false);
                    if let Some(request) = obj.request() {
                        request.start_sas();
                    }
                }));
            self.start_emoji_btn3
                .connect_clicked(clone!(@weak obj => move |button| {
                    let priv_ = imp::IdentityVerificationWidget::from_instance(&obj);
                    button.set_loading(true);
                    priv_.take_screenshot_btn3.set_sensitive(false);
                    if let Some(request) = obj.request() {
                        request.start_sas();
                    }
                }));

            self.scan_qr_code_btn
                .connect_clicked(clone!(@weak obj => move |button| {
                    let priv_ = imp::IdentityVerificationWidget::from_instance(&obj);
                    button.set_loading(true);
                    priv_.start_emoji_btn.set_sensitive(false);
                    if priv_.qr_code_scanner.has_camera() {
                        obj.start_scanning();
                    } else {
                        obj.take_screenshot();
                    }
                }));

            self.take_screenshot_btn2
                .connect_clicked(clone!(@weak obj => move |button| {
                    let priv_ = imp::IdentityVerificationWidget::from_instance(&obj);
                    button.set_loading(true);
                    priv_.start_emoji_btn2.set_sensitive(false);
                    obj.take_screenshot();
                }));

            self.take_screenshot_btn3
                .connect_clicked(clone!(@weak obj => move |button| {
                    let priv_ = imp::IdentityVerificationWidget::from_instance(&obj);
                    button.set_loading(true);
                    priv_.start_emoji_btn3.set_sensitive(false);
                    obj.take_screenshot();
                }));

            self.done_btn.connect_clicked(clone!(@weak obj => move |_| {
                if let Some(request) = obj.request() {
                    if request.mode() == VerificationMode::CurrentSession {
                        obj.activate_action("session.show-content", None);
                    }
                }
            }));

            self.qr_code_scanner
                .connect_code_detected(clone!(@weak obj => move |_, data| {
                    obj.finish_scanning(data);
                }));

            self.qr_code_scanner.connect_notify_local(
                Some("has-camera"),
                clone!(@weak obj => move |_, _| {
                    obj.update_camera_state();
                }),
            );
            obj.update_camera_state();
        }

        fn dispose(&self, obj: &Self::Type) {
            if let Some(request) = obj.request() {
                if let Some(handler) = self.state_handler.take() {
                    request.disconnect(handler);
                }

                if let Some(handler) = self.name_handler.take() {
                    request.user().disconnect(handler);
                }
            }
        }
    }

    impl WidgetImpl for IdentityVerificationWidget {
        fn map(&self, widget: &Self::Type) {
            self.parent_map(widget);
            widget.update_view();
        }
    }
    impl BinImpl for IdentityVerificationWidget {}
}

glib::wrapper! {
    pub struct IdentityVerificationWidget(ObjectSubclass<imp::IdentityVerificationWidget>)
        @extends gtk::Widget, adw::Bin, @implements gtk::Accessible;
}

impl IdentityVerificationWidget {
    pub fn new(request: &IdentityVerification) -> Self {
        glib::Object::new(&[("request", request)])
            .expect("Failed to create IdentityVerificationWidget")
    }

    pub fn request(&self) -> Option<IdentityVerification> {
        let priv_ = imp::IdentityVerificationWidget::from_instance(self);
        priv_.request.borrow().clone()
    }

    pub fn set_request(&self, request: Option<IdentityVerification>) {
        let priv_ = imp::IdentityVerificationWidget::from_instance(self);
        let previous_request = self.request();

        if previous_request == request {
            return;
        }

        self.reset();

        if let Some(previous_request) = previous_request {
            if let Some(handler) = priv_.state_handler.take() {
                previous_request.disconnect(handler);
            }

            if let Some(handler) = priv_.name_handler.take() {
                previous_request.user().disconnect(handler);
            }
        }

        if let Some(ref request) = request {
            let handler = request.connect_notify_local(
                Some("state"),
                clone!(@weak self as obj => move |_, _| {
                    obj.update_view();
                }),
            );

            priv_.state_handler.replace(Some(handler));

            let handler = request.user().connect_notify_local(
                Some("display-name"),
                clone!(@weak self as obj => move |_, _| {
                    obj.init_mode();
                }),
            );

            priv_.name_handler.replace(Some(handler));
        }

        priv_.request.replace(request);
        self.init_mode();
        self.update_view();
        self.notify("request");
    }

    fn reset(&self) {
        let priv_ = imp::IdentityVerificationWidget::from_instance(self);
        priv_.accept_btn.set_loading(false);
        priv_.accept_btn.set_sensitive(true);
        priv_.decline_btn.set_sensitive(true);
        priv_.scan_qr_code_btn.set_loading(false);
        priv_.scan_qr_code_btn.set_sensitive(true);
        priv_.emoji_not_match_btn.set_loading(false);
        priv_.emoji_not_match_btn.set_sensitive(true);
        priv_.emoji_match_btn.set_loading(false);
        priv_.emoji_match_btn.set_sensitive(true);
        priv_.start_emoji_btn.set_loading(false);
        priv_.start_emoji_btn.set_sensitive(true);
        priv_.start_emoji_btn2.set_loading(false);
        priv_.start_emoji_btn2.set_sensitive(true);
        priv_.start_emoji_btn3.set_loading(false);
        priv_.start_emoji_btn3.set_sensitive(true);
        priv_.take_screenshot_btn2.set_loading(false);
        priv_.take_screenshot_btn2.set_sensitive(true);
        priv_.take_screenshot_btn3.set_loading(false);
        priv_.take_screenshot_btn3.set_sensitive(true);

        self.clean_emoji();
    }

    fn clean_emoji(&self) {
        let priv_ = imp::IdentityVerificationWidget::from_instance(self);

        while let Some(child) = priv_.emoji_row_1.first_child() {
            priv_.emoji_row_1.remove(&child);
        }

        while let Some(child) = priv_.emoji_row_2.first_child() {
            priv_.emoji_row_2.remove(&child);
        }
    }

    pub fn accept(&self) {
        if let Some(request) = self.request() {
            request.accept();
        }
    }

    pub fn decline(&self) {
        if let Some(request) = self.request() {
            request.cancel(true);
        }
    }

    fn update_view(&self) {
        let priv_ = imp::IdentityVerificationWidget::from_instance(self);
        if let Some(request) = self.request() {
            match request.state() {
                VerificationState::Requested => {
                    priv_.main_stack.set_visible_child_name("accept-request");
                }
                VerificationState::RequestSend => {
                    priv_
                        .main_stack
                        .set_visible_child_name("wait-for-other-party");
                }
                VerificationState::QrV1Show => {
                    if let Some(qrcode) = request.qr_code() {
                        priv_.qrcode.set_qrcode(qrcode.clone());
                        priv_.main_stack.set_visible_child_name("qrcode");
                    } else {
                        warn!("Failed to get qrcode for QrVerification");
                        request.start_sas();
                    }
                }
                VerificationState::QrV1Scan => {
                    self.start_scanning();
                }
                VerificationState::SasV1 => {
                    self.clean_emoji();
                    match request.sas_data().unwrap() {
                        SasData::Emoji(emoji) => {
                            for (index, emoji) in emoji.iter().enumerate() {
                                if index < 4 {
                                    priv_.emoji_row_1.append(&Emoji::new(emoji));
                                } else {
                                    priv_.emoji_row_2.append(&Emoji::new(emoji));
                                }
                            }
                        }
                        SasData::Decimal((a, b, c)) => {
                            let container = gtk::Box::builder()
                                .spacing(24)
                                .css_classes(vec!["emoji".to_string()])
                                .build();
                            container.append(&gtk::Label::builder().label(&a.to_string()).build());
                            container.append(&gtk::Label::builder().label(&b.to_string()).build());
                            container.append(&gtk::Label::builder().label(&c.to_string()).build());
                            priv_.emoji_row_1.append(&container);
                        }
                    }
                    priv_.main_stack.set_visible_child_name("emoji");
                }
                VerificationState::Completed => {
                    priv_.main_stack.set_visible_child_name("completed");
                }
                VerificationState::Cancelled
                | VerificationState::Dismissed
                | VerificationState::Error
                | VerificationState::Passive => {}
            }
        }
    }

    fn start_scanning(&self) {
        spawn!(clone!(@weak self as obj => async move {
            let priv_ = imp::IdentityVerificationWidget::from_instance(&obj);
            if priv_.qr_code_scanner.start().await {
                priv_.main_stack.set_visible_child_name("scan-qr-code");
            } else {
                priv_.main_stack.set_visible_child_name("no-camera");
            }
        }));
    }

    fn take_screenshot(&self) {
        spawn!(clone!(@weak self as obj => async move {
            let root = obj.root().unwrap();
            if let Some(code) = screenshot::capture(&root).await {
                obj.finish_scanning(code);
            } else {
                obj.reset();
            }
        }));
    }

    fn finish_scanning(&self, data: QrVerificationData) {
        let priv_ = imp::IdentityVerificationWidget::from_instance(self);
        priv_.qr_code_scanner.stop();
        if let Some(request) = self.request() {
            request.scanned_qr_code(data);
        }
        priv_.main_stack.set_visible_child_name("qr-code-scanned");
    }

    fn update_camera_state(&self) {
        let priv_ = imp::IdentityVerificationWidget::from_instance(self);
        if priv_.qr_code_scanner.has_camera() {
            priv_
                .scan_qr_code_btn
                .set_label(&gettext("Scan QR code with this session"))
        } else {
            priv_
                .scan_qr_code_btn
                .set_label(&gettext("Take a Screenshot of a Qr Code"))
        }
    }

    fn init_mode(&self) {
        let priv_ = imp::IdentityVerificationWidget::from_instance(self);
        let request = if let Some(request) = self.request() {
            request
        } else {
            return;
        };

        match request.mode() {
            VerificationMode::CurrentSession => {
                // label1 and label2 won't be shown
                priv_
                    .label2
                    .set_label(&gettext("Verify the new session with the current session."));
                priv_.label3.set_label(&gettext("Verify Session"));
                priv_.label4.set_label(&gettext("Scan the Qr code with this session from another session logged into this account."));
                priv_.label5.set_label(&gettext("You scanned to qr code successfully. You may need to confirm the verification in the other session."));
                priv_.label6.set_label(&gettext("Verify Session"));
                priv_
                    .label7
                    .set_label(&gettext("Select an option to verify the new session."));
                priv_.label8.set_label(&gettext("Verify Session"));
                priv_.label9.set_label(&gettext(
                    "Scan this qr code with the newly logged in session.",
                ));
                priv_.label10.set_label(&gettext("Verify Session"));
                priv_.label11.set_label(&gettext(
                    "Check if the same emoji appear in the same order on the other device.",
                ));
                priv_.label12.set_label(&gettext("Request Complete"));
                priv_.label13.set_label(&gettext(
                    "This session is ready to send and receive secure messages.",
                ));
                priv_.done_btn.set_label(&gettext("Get Started"));
            }
            VerificationMode::OtherSession => {
                priv_
                    .label1
                    .set_label(&gettext("Login Request From Another Session"));
                priv_
                    .label2
                    .set_label(&gettext("Verify the new session with the current session."));
                priv_.label3.set_label(&gettext("Verify Session"));
                priv_.label4.set_label(&gettext("Scan the Qr code with this session from another session logged into this account."));
                priv_.label5.set_label(&gettext("You scanned to qr code successfully. You may need to confirm the verification in the other session."));
                priv_.label6.set_label(&gettext("Verify Session"));
                priv_
                    .label7
                    .set_label(&gettext("Select an option to verify the new session."));
                priv_.label8.set_label(&gettext("Verify Session"));
                priv_.label9.set_label(&gettext(
                    "Scan this qr code with the newly logged in session.",
                ));
                priv_.label10.set_label(&gettext("Verify Session"));
                priv_.label11.set_label(&gettext(
                    "Check if the same emoji appear in the same order on the other device.",
                ));
                priv_.label12.set_label(&gettext("Request Complete"));
                priv_.label13.set_label(&gettext(
                    "The new session is now ready to send and receive secure messages.",
                ));
                priv_.label14.set_label(&gettext("Get Another Device"));
                priv_.label15.set_label(&gettext(
                    "Accept the verification request from another session or device.",
                ));
            }
            VerificationMode::User => {
                let name = request.user().display_name();
                priv_.label1.set_markup(&gettext("Verification Request"));
                priv_
                    .label2
                    .set_markup(&gettext!("<b>{}</b> asked do be verified. Verifying an user increases the security of the conversation.", name));
                priv_.label3.set_markup(&gettext("Verification Request"));
                priv_.label4.set_markup(&gettext!(
                    "Scan the Qr code shown on the device of <b>{}</b>.",
                    name
                ));
                priv_.label5.set_markup(&gettext!("You scanned the Qr code successfully. <b>{}</b> may need to confirm the verification.", name));
                priv_.label6.set_markup(&gettext("Verification Request"));
                priv_
                    .label7
                    .set_markup(&gettext!("Select an option to verify <b>{}</b>", name));
                priv_.label8.set_markup(&gettext("Verification Request"));
                priv_.label9.set_markup(&gettext(
                    "Ask <b>{}</b> to scan this Qr code with there device.",
                ));
                priv_.label10.set_markup(&gettext("Verification Request"));
                priv_.label11.set_markup(&gettext!(
                    "Ask <b>{}</b> if they see the following emoji appear in the same order on there screen.",
                    name
                ));
                priv_.label12.set_markup(&gettext("Verification Complete"));
                priv_.label13.set_markup(&gettext!("<b>{}</b>is now verified and you can now be sure that your comunication will be private.", name));
            }
        }
    }
}
