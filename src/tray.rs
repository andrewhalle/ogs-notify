use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

use cpp_core::{Ptr, Ref, StaticUpcast};
use qt_core::{slot, QBox, QObject, QString};
use qt_gui::QIcon;
use qt_widgets::{QApplication, QSystemTrayIcon, SlotOfQIcon};

struct TrayIcon(QBox<QSystemTrayIcon>);

pub struct TrayHandle(QBox<SlotOfQIcon>);

unsafe impl Send for TrayHandle {}

impl TrayHandle {
    pub fn set(&mut self, filename: PathBuf) {
        unsafe {
            let icon = QIcon::from_q_string(&QString::from_std_str(filename.to_str().unwrap()));

            self.0.slot(&icon);
        }
    }
}

impl StaticUpcast<QObject> for TrayIcon {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.0.as_ptr().static_upcast()
    }
}

impl TrayIcon {
    #[slot(SlotOfQIcon)]
    unsafe fn change_icon(self: &Rc<Self>, icon: Ref<QIcon>) {
        self.0.set_icon(icon);
    }
}

pub fn run_tray(filename: PathBuf) -> TrayHandle {
    let (tx, rx) = mpsc::channel();

    unsafe {
        thread::spawn(move || {
            QApplication::init(|_| {
                let icon = QSystemTrayIcon::new();
                icon.set_icon(&QIcon::from_q_string(&QString::from_std_str(
                    filename.to_str().unwrap(),
                )));
                icon.show();

                let icon = Rc::new(TrayIcon(icon));
                let slot = TrayHandle(icon.slot_change_icon());

                tx.send(slot).unwrap();

                QApplication::exec()
            });
        });
    }

    rx.recv().unwrap()
}
