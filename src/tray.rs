use tao::event_loop::EventLoopProxy;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

pub enum TrayUserEvent {
    TrayIconEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
}

pub struct TrayConfig {
    pub tooltip: String,
    pub icon_svg_bytes: &'static [u8],
    pub custom_menu_items: Vec<(String, String)>, // (id, label)
}

pub fn create_tray(proxy: EventLoopProxy<TrayUserEvent>, config: TrayConfig) -> TrayIcon {
    let proxy_clone = proxy.clone();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = proxy_clone.send_event(TrayUserEvent::TrayIconEvent(event));
    }));

    let proxy_clone2 = proxy.clone();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy_clone2.send_event(TrayUserEvent::MenuEvent(event));
    }));

    let menu = Menu::new();
    let open_item = MenuItem::with_id("open", "Открыть", true, None);
    let _ = menu.append_items(&[&open_item]);

    // Добавляем кастомные пункты
    for (id, label) in &config.custom_menu_items {
        let item = MenuItem::with_id(id, label, true, None);
        let _ = menu.append_items(&[&item]);
    }

    let close_item = MenuItem::with_id("close", "Закрыть", true, None);
    let _ = menu.append_items(&[&close_item]);

    let icon = load_svg_icon(config.icon_svg_bytes);

    TrayIconBuilder::new()
        .with_tooltip(config.tooltip)
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .build()
        .unwrap()
}

fn load_svg_icon(svg_data: &[u8]) -> Icon {
    let opt = usvg::Options::default();
    let rtree = usvg::Tree::from_data(svg_data, &opt).expect("Failed to parse SVG");

    let size = rtree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height()).unwrap();

    resvg::render(&rtree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    Icon::from_rgba(pixmap.data().to_vec(), size.width(), size.height()).unwrap()
}
