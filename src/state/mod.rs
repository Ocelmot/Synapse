use std::collections::HashMap;

use spider_client::{SpiderClient, message::{Message, DatasetMessage, UiMessage, DatasetPath, UiInput, UiPageManager, RouterMessage, DatasetData, DirectoryEntry}, Relation};



mod ui;


pub struct State {
    client: SpiderClient,
    page_mgr: UiPageManager,

    // maps relations to indices in the contacts dataset
    directory: HashMap<Relation, DirectoryEntry>,
    contacts: HashMap<Relation, usize>,

    messages: HashMap<Relation, Vec<DatasetData>>,
    current_recp: Option<Relation>,
}

impl State {


    pub async fn new(client: SpiderClient) -> Self{
        let id = client.self_relation().id;
        let page_mgr = UiPageManager::new(id.clone(), "Synapse");

        let mut new = Self {
            client,
            page_mgr,
            directory: HashMap::new(),
            contacts: HashMap::new(),
            messages: HashMap::new(),
            current_recp: None,
        };

        new.init().await;

        new
    }

    pub async fn run(&mut self) {
        loop {
            match self.client.recv().await {
                Some(Message::Ui(msg)) => self.ui_handler(msg).await,
                Some(Message::Dataset(msg)) => self.dataset_handler(msg).await,
                Some(Message::Router(msg)) => self.router_handler(msg).await,
                None => break, //  done!
            }
        }
    }

    async fn init(&mut self) {
        // identify peripheral
        let msg = RouterMessage::SetIdentityProperty("name".into(), "Synapse".into());
        let msg = Message::Router(msg);
        self.client.send(msg).await;

        // subscribe to directory
        let msg = RouterMessage::SubscribeDir;
        let msg = Message::Router(msg);
        self.client.send(msg).await;

        // subscribe to incoming messages (type="chat")
        let msg = RouterMessage::Subscribe("chat".into());
        let msg = Message::Router(msg);
        self.client.send(msg).await;
        
        // contacts: {id: base_64 of relation, name: from directory}
        // let contacts_path = DatasetPath::new_private(vec!["contacts".into()]);
        // let msg = DatasetMessage::Subscribe { path: contacts_path };
        // let msg = Message::Dataset(msg);
        // self.client.send(msg).await;

        self.show_list().await;
    }

    async fn dataset_handler(&mut self, msg: DatasetMessage) {
        if let DatasetMessage::Dataset { path, data } = msg {
            // dataset has changed, update internal structs
            let parts = path.parts();
            if parts.get(0) != Some(&String::from("contacts")) {
                return; // all messages are in contacts or sub-datasets
            }
            match parts.get(1){
                Some(id) => {
                    // this is a subdataset, process messages
                    let rel = Relation::from_base64(id.into()).unwrap();

                    // if path is too long, remove first element
                    if data.len() > 20{
                        let msg = Message::Dataset(DatasetMessage::DeleteElement {
                            path: path.clone(),
                            id: 0,
                        });
                        self.client.send(msg).await;
                    }
                    self.messages.insert(rel, data);
                },
                None => {},
            }
        }
    }

    async fn ui_handler(&mut self, msg: UiMessage) {
        match msg {
            UiMessage::Subscribe => {}
            UiMessage::Pages(_) => {}
            UiMessage::GetPage(_) => {}
            UiMessage::Page(_) => {}
            UiMessage::UpdateElementsFor(_, _) => {}
            UiMessage::InputFor(_, _, _, _) => {}
            UiMessage::SetPage(_) => {}
            UiMessage::ClearPage => {}
            UiMessage::UpdateElements(_) => {}
            UiMessage::Input(element_id, dataset_ids, change) => {
                match element_id.as_str() {
                    "contact" => {
                        // find selected contact
                        // set ui to that page
                        println!("pressed contact");
                        let index = dataset_ids.get(0).expect("always should be in a dataset");
                        for (rel, idx) in &self.contacts {
                            if index == idx{
                                // found index, set page to this
                                self.show_msgs(rel.clone()).await;

                                break;
                            }
                        }
                    }
                    "back" => {
                        // switch back to contacts list
                        self.current_recp = None;
                        self.show_list().await;
                    }
                    "message" => {
                        // send message to selected recp
                        if let UiInput::Text(text) = change{
                            if let Some(rel) = &self.current_recp{
                                // append to dataset (to render)
                                let chat_path = DatasetPath::new_private(vec!["contacts".into(), rel.sha256()]);
                                let chat = DatasetData::Map({
                                    HashMap::from([
                                        ("self".into(), DatasetData::String(text.clone())),
                                        ("other".into(), DatasetData::String("".into()))
                                    ])
                                });
                                let msg = Message::Dataset(DatasetMessage::Append {
                                    path: chat_path,
                                    data: chat,
                                });
                                self.client.send(msg).await;

                                // send message to peer
                                let recps = vec![rel.clone()];
                                let chat = DatasetData::String(text);
                                let msg = RouterMessage::SendEvent("chat".into(), recps, chat);
                                let msg = Message::Router(msg);
                                self.client.send(msg).await;
                            }
                        }
                    },
                    _ => return,
                }
            }
            UiMessage::Dataset(_, _) => {}
        }
    }

    async fn router_handler(&mut self, msg: RouterMessage){
        match msg {
            RouterMessage::SendEvent(_, _, _) => {},
            RouterMessage::Event(msg_type, from, data) => {
                // a new chat message has arrived
                if msg_type != "chat"{
                    return;
                }

                if self.contacts.contains_key(&from) {
                    // add message to dataset
                    let path = DatasetPath::new_private(vec!["contacts".into(), from.sha256()]);
                    let data = if let DatasetData::String(s) = data {
                        DatasetData::Map({
                            HashMap::from([
                                ("self".into(), DatasetData::String("".into())),
                                ("other".into(), DatasetData::String(s.clone()))
                            ])
                        })
                    } else {
                        return;
                    };
                    let msg = DatasetMessage::Append { path, data };
                    let msg = Message::Dataset(msg);
                    self.client.send(msg).await;
                }
            },
            RouterMessage::Subscribe(_) => {},
            RouterMessage::Unsubscribe(_) => {},
            RouterMessage::SubscribeDir => {},
            RouterMessage::UnsubscribeDir => {},
            RouterMessage::AddIdentity(entry) => {
                // a directory entry has changed, update the dataset
                let rel = entry.relation().clone();
                if rel.is_peripheral() {
                    return;
                }
                self.directory.insert(rel.clone(), entry.clone());
                let next_index = self.contacts.len();
                let index = self.contacts.entry(rel.clone()).or_insert(next_index);

                // set dataset item to this
                let name = entry.get("nickname")
                    .or(entry.get("name"))
                    .unwrap_or(&String::from("NoName")).clone();

                let contact_path = DatasetPath::new_private(vec!["contacts".into()]);
                let new_data = DatasetData::Map({
                    HashMap::from([
                        ("id".into(), DatasetData::String(rel.to_base64())),
                        ("name".into(), DatasetData::String(name))
                    ])
                });
                let msg = DatasetMessage::SetElement { path: contact_path, data: new_data, id: *index };
                let msg = Message::Dataset(msg);

                self.client.send(msg).await;
            },
            RouterMessage::RemoveIdentity(_) => {},
            RouterMessage::SetIdentityProperty(_, _) => {},
        }
    }
}
