use gtk::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type RefMap = Rc<RefCell<HashMap<String, String>>>;

fn main() {
    gtk::init().unwrap();
    let mut med = Med::default();

    med.create_box();
    med.wire();
    gtk::main();
}

struct Med {
    btns: HashMap<String, Button>,
    boxes: HashMap<String, Box>,
    wins: HashMap<String, Window>,
    database: RefMap,
}

impl Default for Med {
    fn default() -> Self {
        let database = Rc::new(RefCell::new(Self::parse_database()));
        Self {
            btns: HashMap::new(),
            boxes: HashMap::new(),
            wins: HashMap::new(),
            database,
        }
    }
}

impl Med {
    fn create_box(&mut self) {
        let add_btn = Button::new_with_label("Add");
        let rm_btn = Button::new_with_label("Rm");

        let hbox = Box::new(Orientation::Horizontal, 10);
        hbox.add(&add_btn);
        hbox.add(&rm_btn);

        let vbox = Box::new(Orientation::Vertical, 10);
        vbox.add(&hbox);
        for name in self.database.borrow().keys() {
            Self::sig_patient_add(name.to_string(), &vbox, &self.database, None)
        }

        let win = Window::new(WindowType::Toplevel);
        win.set_title("Med");
        win.add(&vbox);
        win.show_all();

        self.btns.insert("Add".to_string(), add_btn);
        self.btns.insert("rm".to_string(), rm_btn);

        self.boxes.insert("hbox".to_string(), hbox);
        self.boxes.insert("vbox".to_string(), vbox);

        self.wins.insert("Win".to_string(), win);
    }
    fn wire(&mut self) {
        let vbox = self.boxes["vbox"].clone();
        let database = self.database.clone();
        self.btns["Add"].connect_clicked(move |_btn| {
            let entry_text = Entry::new();
            let entry_win = Window::new(WindowType::Toplevel);
            entry_win.add(&entry_text);
            entry_win.set_title("Name");

            // clone ref
            let vbox = vbox.clone();
            let ew = entry_win.clone();
            let database = database.clone();
            //

            entry_text.connect_activate(move |et| {
                Self::sig_patient_add(
                    et.get_text().unwrap().to_string(),
                    &vbox,
                    &database,
                    Some(&ew),
                );
            });
            entry_win.show_all();
        });
    }
    // parse
    fn parse_database() -> HashMap<String, String> {
        use std::fs;
        use std::io::Read;

        let mut map_base = HashMap::new();
        let med_dir = {
            let mut d = dirs::config_dir().unwrap();
            d.push("Med");
            d
        };
        if !med_dir.is_dir() {
            fs::create_dir(&med_dir).unwrap();
        }

        for patient_hash in fs::read_dir(med_dir).unwrap() {
            let data_file = {
                let mut data = patient_hash.unwrap().path();
                data.push("data");
                data
            };
            let data = {
                let mut data = String::new();
                let mut d = fs::File::open(data_file).unwrap();
                d.read_to_string(&mut data).unwrap();
                data.to_string()
            };
            let data: Vec<String> = data.lines().map(|l| l.to_string()).collect();
            let patient_name = data[0].clone();
            let patient_data: String = data.into_iter().skip(1).collect();
            map_base.insert(patient_name, patient_data);
        }
        map_base
    }
    // signals
    fn sig_patient_add(patient_name: String, vbox: &Box, database: &RefMap, ew: Option<&Window>) {
        let btn = Button::new_with_label(&patient_name);

        // clone ref
        let database = database.clone();

        btn.connect_clicked(move |_btn| {
            // clone ref
            let database_c = database.clone();

            let patient_data = database
                .borrow_mut()
                .entry(patient_name.clone())
                .or_insert_with(String::new)
                .clone();

            let entry_text = Entry::new();
            let save_btn = Button::new_with_label("Save");
            let vbox = Box::new(Orientation::Vertical, 10);

            vbox.pack_start(&entry_text, true, true, 10);
            vbox.pack_start(&save_btn, false, false, 10);

            let entry_win = Window::new(WindowType::Toplevel);
            entry_win.add(&vbox);
            entry_win.set_title(&patient_name);
            entry_text.set_text(&patient_data);

            // clone ref
            let patient_name = RefCell::new(patient_name.clone());
            let ew = entry_win.clone();
            let et = entry_text.clone();

            save_btn.connect_clicked(move |_| {
                let name = patient_name.borrow().to_string();
                let data = et.get_text().unwrap().to_string();
                database_c.borrow_mut().insert(name, data);
                Self::sig_save_patient_data(&database_c);
                ew.destroy();
            });

            entry_win.show_all();
        });

        vbox.add(&btn);
        vbox.show_all();

        if let Some(ew) = ew {
            ew.destroy()
        };
    }
    fn sig_save_patient_data(database: &RefMap) {
        use std::io::*;

        for (p_name, p_data) in database.borrow().iter() {
            let code: String = {
                let hash: [u8; 16] = md5::compute(p_name).into();
                hash.iter().fold(0, |acc, x| acc + *x as usize).to_string()
            };

            let p_dir: std::path::PathBuf = {
                let mut med_dir = Self::med_dir();
                med_dir.push(code);
                med_dir
            };

            if !std::path::Path::exists(&p_dir) {
                std::fs::create_dir(&p_dir).unwrap();
            }

            let data = {
                let mut d = p_dir;
                d.push("data");
                d
            };

            let mut data = std::fs::File::create(data).unwrap();
            let name_and_data = {
                let mut t = p_name.clone();
                t.push('\n');
                t.push_str(p_data.as_str());
                t
            };
            write!(data, "{}", name_and_data).unwrap();
        }
    }

    // helpers
    fn med_dir() -> std::path::PathBuf {
        let mut med_dir = dirs::config_dir().unwrap();
        med_dir.push("Med");
        med_dir
    }
}
