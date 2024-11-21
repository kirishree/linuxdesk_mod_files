#[cfg(target_os = "linux")]
use crate::ipc::start_pa;
use reqwest;
use serde_json::json;
use serde_json::Value as JsonValue;
use crate::ui_cm_interface::{start_ipc, ConnectionManager, InvokeUiCM};
use hbb_common::{
    config::Config,
};

use hbb_common::{allow_err, log};
use sciter::{make_args, Element, Value, HELEMENT};
use std::sync::Mutex;
use std::{ops::Deref, sync::Arc};

#[derive(Clone, Default)]
pub struct SciterHandler {
    pub element: Arc<Mutex<Option<Element>>>,
}

impl InvokeUiCM for SciterHandler {
    fn add_connection(&self, client: &crate::ui_cm_interface::Client) {
        self.call(
            "addConnection",
            &make_args!(
                client.id,
                client.is_file_transfer,
                client.port_forward.clone(),
                client.peer_id.clone(),
                client.name.clone(),
                client.authorized,
                client.keyboard,
                client.clipboard,
                client.audio,
                client.file,
                client.restart,
                client.recording
            ),
        );
    }

    fn remove_connection(&self, id: i32, close: bool) {
        self.call("removeConnection", &make_args!(id, close));
        if crate::ui_cm_interface::get_clients_length().eq(&0) {
            crate::platform::quit_gui();
        }
        crate::ui_interface::update_temporary_password();
    }

    fn new_message(&self, id: i32, text: String) {
        self.call("newMessage", &make_args!(id, text));
    }

    fn change_theme(&self, _dark: String) {
        // TODO
    }

    fn change_language(&self) {
        // TODO
    }

    fn show_elevation(&self, show: bool) {
        self.call("showElevation", &make_args!(show));
    }
}

impl SciterHandler {
    #[inline]
    fn call(&self, func: &str, args: &[Value]) {
        if let Some(e) = self.element.lock().unwrap().as_ref() {
            allow_err!(e.call_method(func, &super::value_crash_workaround(args)[..]));
        }
    }
}

pub struct SciterConnectionManager(ConnectionManager<SciterHandler>);

impl Deref for SciterConnectionManager {
    type Target = ConnectionManager<SciterHandler>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SciterConnectionManager {
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        std::thread::spawn(start_pa);
        let cm = ConnectionManager {
            ui_handler: SciterHandler::default(),
        };
        let cloned = cm.clone();
        std::thread::spawn(move || start_ipc(cloned));
        SciterConnectionManager(cm)
    }

    fn get_icon(&mut self) -> String {
        crate::get_icon()
    }

    fn check_click_time(&mut self, id: i32) {
        crate::ui_cm_interface::check_click_time(id);
    }

    fn get_click_time(&self) -> f64 {
        crate::ui_cm_interface::get_click_time() as _
    }

    fn switch_permission(&self, id: i32, name: String, enabled: bool) {
        crate::ui_cm_interface::switch_permission(id, name, enabled);
    }

    fn close(&self, id: i32) {
        crate::ui_cm_interface::close(id);
        crate::ui_interface::update_temporary_password();
    }

    fn remove_disconnected_connection(&self, id: i32) {
        crate::ui_cm_interface::remove(id);
    }

    fn quit(&self) {
        crate::platform::quit_gui();
    }

    fn authorize(&self, id: i32) {
        crate::ui_cm_interface::authorize(id);
    }

    fn send_msg(&self, id: i32, text: String) {
        crate::ui_cm_interface::send_chat(id, text);
    }

    fn t(&self, name: String) -> String {
        crate::client::translate(name)
    }

    fn can_elevate(&self) -> bool {
        crate::ui_cm_interface::can_elevate()
    }

    fn elevate_portable(&self, id: i32) {
        crate::ui_cm_interface::elevate_portable(id);
    }

    fn get_option(&self, key: String) -> String {
        crate::ui_interface::get_option(key)
    }
      
    fn pswrdsend(&mut self, con_name: String, con_id: String, connid: i32) {
        let server_ip_auth = hbb_common::config::Config::get_reachdesk_serverip();
        let otp_service_url = format!("http://{}:3010/getdata", server_ip_auth);    
        let data = json!({
            "otp": crate::ui_interface::temporary_password(),
            "id": hbb_common::config::Config::get_id(),
            "remote_user_name": con_name,
            "remote_user_id": con_id,
            "user_mail_id": crate::ui_interface::get_email_id(),
            "permanent_password":  crate::ui_interface::permanent_password(),
            "login_id": crate::ui_interface::get_login_id(),
            "login_password": crate::ui_interface::get_login_password()
        });
    
        // Use the `post` function to send a POST request
        let response = reqwest::blocking::Client::new()
            .post(otp_service_url)
            .json(&data)
            .send()
            .expect("Failed to send request");
        // Deserialize the JSON response using a generic Value type
        let json_response1: JsonValue = response.json().expect("Failed to deserialize JSON");

        // Deserialize the specific fields from the Value type
         
        let message_block = json_response1["block"].as_str().expect("Missing message field").to_string();
        if message_block == "true"{
            crate::ui_cm_interface::close(connid);
            crate::ui_interface::update_temporary_password();
        }
    }
}

impl sciter::EventHandler for SciterConnectionManager {
    fn attached(&mut self, root: HELEMENT) {
        *self.ui_handler.element.lock().unwrap() = Some(Element::from(root));
    }

    sciter::dispatch_script_call! {
        fn t(String);
        fn check_click_time(i32);
        fn get_click_time();
        fn get_icon();
        fn close(i32);
        fn remove_disconnected_connection(i32);
        fn quit();
        fn authorize(i32);
        fn switch_permission(i32, String, bool);
        fn send_msg(i32, String);
        fn can_elevate();
        fn elevate_portable(i32);
        fn get_option(String);
        fn pswrdsend(String, String, i32);     
        
    }
}
