use gtk::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::rc::Rc;

type RefMap<U, T> = Rc<RefCell<HashMap<U, T>>>;

fn main() {
    gtk::init().unwrap();
    let mut med = Med::default();

    med.build_gui();
    med.wire();
    gtk::main();
}

struct Med {
    btns: HashMap<String, Button>,
    entrys: HashMap<String, Entry>,
    boxes: HashMap<String, Box>,
    wins: HashMap<String, Window>,
    database: RefMap<String, PathBuf>,
    patients_btns: RefMap<String, Button>,
}

impl Default for Med {
    fn default() -> Self {
        let database = Rc::new(RefCell::new(Self::parse_database()));
        Self {
            btns: HashMap::new(),
            entrys: HashMap::new(),
            boxes: HashMap::new(),
            wins: HashMap::new(),
            patients_btns: Rc::new(RefCell::new(HashMap::new())),
            database,
        }
    }
}

impl Med {
    fn build_gui(&mut self) {
        let add_btn = Button::new_with_label("Add");
        let rm_btn = Button::new_with_label("Rm");
        let search_bar = Entry::new();

        let hbox = Box::new(Orientation::Horizontal, 10);
        hbox.add(&add_btn);
        hbox.add(&rm_btn);

        let vbox = Box::new(Orientation::Vertical, 10);
        vbox.add(&hbox);
        vbox.add(&search_bar);
        for name in self.database.borrow().keys() {
            Self::sig_patient_add(&self.patients_btns, name.to_string(), &vbox, None, None)
        }

        let win = Window::new(WindowType::Toplevel);
        win.set_title("Med");
        win.add(&vbox);
        win.show_all();

        self.btns.insert("add".to_string(), add_btn);
        self.btns.insert("rm".to_string(), rm_btn);
        self.entrys.insert("search".to_string(), search_bar);

        self.boxes.insert("hbox".to_string(), hbox);
        self.boxes.insert("vbox".to_string(), vbox);

        self.wins.insert("Win".to_string(), win);
    }

    fn wire(&mut self) {
        self.sig_search_patient();
        self.sig_add_patients_btn();
        self.sig_remove_patient_btn();
    }

    // parse
    fn parse_database() -> HashMap<String, PathBuf> {
        use std::fs;

        let mut map_base = HashMap::new();
        let med_dir = Self::med_dir();

        if !Self::med_dir().is_dir() {
            fs::create_dir(&med_dir).unwrap();
        }

        for patient_dir in fs::read_dir(med_dir).unwrap() {
            let patient_dir = patient_dir.unwrap().path();
            let patient_name = Self::read_patient_data(&patient_dir).0.unwrap();
            map_base.insert(patient_name, patient_dir);
        }
        map_base
    }

    // signals
    fn sig_search_patient(&mut self) {
        // clone ref
        let search_bar = self.entrys["search"].clone();
        let vbox = self.boxes["vbox"].clone();
        let p_b = self.patients_btns.clone();

        search_bar.connect_property_text_length_notify(move |search_bar| {
            let search = search_bar.get_text().unwrap().to_string();
            let mut visible = vec![];
            let p_b = p_b.borrow();
            p_b.keys().for_each(|p_name| {
                if p_name.contains(&search) {
                    let p_btn = &p_b[p_name];
                    visible.push(p_btn);
                }
            });
            for c in vbox.get_children().iter().skip(2) {
                vbox.remove(c);
            }
            for v in visible {
                vbox.add(v);
            }
        });
    }

    fn sig_add_patients_btn(&mut self) {
        // clone ref
        let vbox = self.boxes["vbox"].clone();
        let patients_btns = self.patients_btns.clone();
        let database = self.database.clone();
        //
        self.btns["add"].connect_clicked(move |_btn| {
            let entry_text = Entry::new();
            let entry_win = Window::new(WindowType::Toplevel);
            entry_win.add(&entry_text);
            entry_win.set_title("Name");

            // clone ref
            let vbox = vbox.clone();
            let ew = entry_win.clone();
            let patients_btns = patients_btns.clone();
            let database = database.clone();
            //

            entry_text.connect_activate(move |et| {
                Self::sig_patient_add(
                    &patients_btns,
                    et.get_text().unwrap().to_string(),
                    &vbox,
                    Some(&database),
                    Some(&ew),
                );
            });
            entry_win.show_all();
        });
    }

    fn sig_patient_add(
        patients_btns: &RefMap<String, Button>,
        patient_name: String,
        vbox: &Box,
        database: Option<&RefMap<String, PathBuf>>,
        ew: Option<&Window>,
    ) {
        let btn = Button::new_with_label(&patient_name);

        //clone
        let pname = patient_name.clone();

        let patient_hash: String = {
            let hash: [u8; 16] = md5::compute(patient_name.clone()).into();
            hash.iter().fold(0, |acc, x| acc + *x as usize).to_string()
        };

        let p_dir: std::path::PathBuf = {
            let mut med_dir = Self::med_dir();
            med_dir.push(patient_hash);
            med_dir
        };

        if !std::path::Path::exists(&p_dir) {
            fs::create_dir(&p_dir).unwrap();
        }

        if let Some(database) = database {
            database
                .borrow_mut()
                .insert(patient_name.clone(), p_dir.clone());
        }

        // if no data is present add patient name as data
        // if Self::read_patient_data(&p_dir).0.is_none() {
        //     Self::sig_save_patient_data(&p_dir, patient_name.clone(), "".to_string());
        // }

        btn.connect_clicked(move |_btn| {
            let entry_text = Entry::new();
            let save_btn = Button::new_with_label("Save");
            let vbox = Box::new(Orientation::Vertical, 10);

            vbox.pack_start(&entry_text, true, true, 10);
            vbox.pack_start(&save_btn, false, false, 10);

            let entry_win = Window::new(WindowType::Toplevel);
            entry_win.add(&vbox);
            entry_win.set_title(&patient_name);
            entry_text.set_text(&Self::read_patient_data(&p_dir).1);

            // clone ref
            let patient_name = RefCell::new(patient_name.clone());
            let ew = entry_win.clone();
            let et = entry_text.clone();

            let p_dir = p_dir.clone();

            save_btn.connect_clicked(move |_| {
                let p_name = patient_name.borrow().to_string();
                let p_data = et.get_text().unwrap().to_string();
                Self::sig_save_patient_data(&p_dir, p_name, p_data);
                ew.destroy();
            });

            entry_win.show_all();
        });

        vbox.add(&btn);
        patients_btns.borrow_mut().insert(pname, btn);

        vbox.show_all();

        if let Some(ew) = ew {
            ew.destroy()
        };
    }
    fn sig_save_patient_data(p_dir: &PathBuf, p_name: String, p_data: String) {
        if !std::path::Path::exists(&p_dir) {
            std::fs::create_dir(&p_dir).unwrap();
        }

        let data = {
            let mut d = p_dir.clone();
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
    fn sig_remove_patient_btn(&mut self) {
        // ref clone
        let big_vbox = self.boxes["vbox"].clone();
        let patients_btns = self.patients_btns.clone();
        let database = self.database.clone();

        self.btns["rm"].connect_clicked(move |_btn| {
            let entry_text = Entry::new();
            let rm_btn = Button::new_with_label("Remove");
            let vbox = Box::new(Orientation::Vertical, 10);
            let entry_win = Window::new(WindowType::Toplevel);

            vbox.pack_start(&entry_text, false, false, 10);
            vbox.pack_start(&rm_btn, false, false, 10);
            vbox.add(&entry_text);
            vbox.add(&rm_btn);

            entry_win.add(&vbox);
            entry_win.set_title("Remove Patient");
            entry_win.show_all();

            //ref clone
            let patients_btns = patients_btns.clone();
            let et = entry_text.clone();
            let ew = entry_win.clone();
            let big_vbox = big_vbox.clone();
            let database = database.clone();

            rm_btn.connect_clicked(move |_rm_btn| {
                let p_name = et.get_text().unwrap().to_string();
                let p_btn = &patients_btns.borrow()[&p_name];
                let p_dir = &database.borrow()[&p_name];

                let _ = fs::remove_dir_all(&p_dir);

                big_vbox.remove(p_btn);
                ew.destroy();
            });
        });
    }

    // manipulate patient data
    fn read_patient_data(p_dir: &PathBuf) -> (Option<String>, String) {
        let data_file = {
            let mut data = p_dir.clone();
            data.push("data");
            data
        };

        if !std::path::Path::exists(&data_file) {
            return (None, "".to_string());
        }

        let data = {
            let mut data = String::new();
            let mut d = fs::File::open(data_file).unwrap();
            d.read_to_string(&mut data).unwrap();
            data.to_string()
        };
        let data: Vec<String> = data.lines().map(|l| l.to_string()).collect();
        let patient_name = data[0].clone();
        let patient_data: String = data.into_iter().skip(1).collect();
        (Some(patient_name), patient_data)
    }
    // helpers
    fn med_dir() -> std::path::PathBuf {
        let mut med_dir = dirs::config_dir().unwrap();
        med_dir.push("Med");
        med_dir
    }
}
