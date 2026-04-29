use crate::tray::{TrayConfig, TrayUserEvent, create_tray};
use axum::{Router, routing::get};
use std::thread;
use tao::event_loop::{ControlFlow, EventLoopBuilder};

pub struct WintrayAppBuilder {
    tooltip: String,
    icon_svg_bytes: Option<&'static [u8]>,
    router: Option<Router>,
    address: Option<String>,
    custom_menu_items: Vec<(String, String)>,
}

impl Default for WintrayAppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WintrayAppBuilder {
    pub fn new() -> Self {
        Self {
            tooltip: "Wintray Application".into(),
            icon_svg_bytes: None,
            router: None,
            address: None,
            custom_menu_items: Vec::new(),
        }
    }

    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = tooltip.into();
        self
    }

    pub fn with_icon(mut self, icon_svg_bytes: &'static [u8]) -> Self {
        self.icon_svg_bytes = Some(icon_svg_bytes);
        self
    }

    pub fn with_router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    pub fn add_menu_item(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.custom_menu_items.push((id.into(), label.into()));
        self
    }

    pub fn build(self) -> WintrayApp {
        let router = self.router.unwrap_or_else(|| {
            Router::new().route("/", get(|| async { "Wintray App is running" }))
        });
        let address = self.address.unwrap_or_else(|| "127.0.0.1:9876".to_string());

        WintrayApp {
            tray_config: TrayConfig {
                tooltip: self.tooltip,
                icon_svg_bytes: self.icon_svg_bytes.expect("Icon must be set before building (use .with_icon())"),
                custom_menu_items: self.custom_menu_items,
            },
            router,
            address,
        }
    }
}

pub struct WintrayApp {
    tray_config: TrayConfig,
    router: Router,
    address: String,
}

impl WintrayApp {
    pub fn run(self) {
        self.run_with(|_| {});
    }

    pub fn run_with<F>(self, mut custom_handler: F)
    where
        F: FnMut(&str) + 'static,
    {
        let address = self.address.clone();
        let ui_address = address.clone();

        // 1. Запуск сервера
        let router = self.router;
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
                axum::serve(listener, router).await.unwrap();
            });
        });

        // 2. Настройка Event Loop
        let event_loop = EventLoopBuilder::<TrayUserEvent>::with_user_event().build();
        let proxy = event_loop.create_proxy();
        let _tray_icon = create_tray(proxy.clone(), self.tray_config);

        // 3. Запуск цикла
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            if let tao::event::Event::UserEvent(user_event) = event {
                match user_event {
                    TrayUserEvent::TrayIconEvent(tray_event) => {
                        if let tray_icon::TrayIconEvent::Click {
                            button: tray_icon::MouseButton::Left,
                            button_state: tray_icon::MouseButtonState::Up,
                            ..
                        } = tray_event
                        {
                            let _ = open::that(format!("http://{}", ui_address));
                        }
                    }
                    TrayUserEvent::MenuEvent(menu_event) => {
                        if menu_event.id == "open" {
                            let _ = open::that(format!("http://{}", ui_address));
                        } else if menu_event.id == "close" {
                            *control_flow = ControlFlow::Exit;
                        } else {
                            custom_handler(menu_event.id.as_ref());
                        }
                    }
                }
            }
        });
    }
}
