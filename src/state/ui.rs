use spider_client::{message::{UiElementKind, UiElement, UiPath, UiElementContent, UiElementContentPart, Message, UiMessage, DatasetPath}, Relation};

use crate::state::State;






impl State{

    pub async fn show_list(&mut self){
        let id = self.client.self_relation().id;
    
        let contacts_path = DatasetPath::new_private(vec!["contacts".into()]).resolve(id.clone());
        // Set page to show private contacts dataset
        // contact has name, id
        //page is just rows with button with name; need to map id to index
        let mut root = self.page_mgr
            .get_element_mut(&UiPath::root())
            .expect("all pages have a root");
        root.set_kind(UiElementKind::Rows);
        for i in (0..root.children().len()).rev(){
            root.delete_child(i);
        }
        root.append_child(UiElement::from_string("Contacts:"));
        
        root.append_child({
            // contact rows
            let mut rows = UiElement::new(UiElementKind::Rows);
            rows.set_dataset(Some(contacts_path));

            // define row template
            rows.append_child({
                let mut button = UiElement::new(UiElementKind::Button);
                button.set_id("contact");
                let content = UiElementContent::new_data("name".into());
                button.set_content(content);
                button
            });

            rows
        });
    
        drop(root);
    
        self.page_mgr.get_changes();

        let msg = Message::Ui(UiMessage::SetPage(self.page_mgr.get_page().clone()));
        self.client.send(msg).await;
    }
    
    
    
    pub async fn show_msgs(&mut self, rel: Relation){
        let name = match self.directory.get(&rel){
            Some(entry) => {
                entry.get("nickname")
                    .or(entry.get("name"))
                    .unwrap_or(&String::from("NoName")).clone()
            },
            None => return,
        };

        // update current recp
        self.current_recp = Some(rel.clone());
        
        // setup page
        let id = self.client.self_relation().id;
    
        
        
        let mut root = self.page_mgr
            .get_element_mut(&UiPath::root())
            .expect("all pages have a root");
        root.set_kind(UiElementKind::Rows);
        for i in (0..root.children().len()).rev(){
            root.delete_child(i);
        }
        // root.append_child({});

        // back button
        root.append_child({
            let mut back = UiElement::new(UiElementKind::Button);
            back.set_id("back");
            back.set_text("Back");
            back
        });

        // name
        root.append_child(UiElement::from_string(name));
        
        // messages
        let msgs_path = DatasetPath::new_private(vec!["contacts".into(), rel.sha256()]).resolve(id.clone());
        root.append_child({
            let mut messages = UiElement::new(UiElementKind::Rows);
            messages.set_dataset(Some(msgs_path));
            messages.append_child({
                let mut message = UiElement::new(UiElementKind::Columns);
                message.append_child({
                    let mut other_msg = UiElement::new(UiElementKind::Text);
                    other_msg.set_content(UiElementContent::new_data("other".into()));
                    other_msg
                });
                message.append_child(UiElement::new(UiElementKind::Spacer));
                message.append_child({
                    let mut self_msg = UiElement::new(UiElementKind::Text);
                    self_msg.set_content(UiElementContent::new_data("self".into()));
                    self_msg
                });
                message
            });
            messages
        });

        // message entry
        root.append_child({
            let mut msg_entry = UiElement::new(UiElementKind::TextEntry);
            msg_entry.set_text("Send Message:");
            msg_entry.set_id("message");
            msg_entry
        });


        // make all the changes
        drop(root);
    
        self.page_mgr.get_changes();

        let msg = Message::Ui(UiMessage::SetPage(self.page_mgr.get_page().clone()));
        self.client.send(msg).await;

    }
}

