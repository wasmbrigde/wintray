use crate::tray::{TrayConfig, TrayUserEvent, create_tray};
use askama::Template;
use poem::{
    EndpointExt, FromRequest, Route, Server,
    endpoint::{BoxEndpoint, make},
    get,
    listener::{Listener, RustlsConfig, TcpListener},
    web::{Html, Path},
};
use rust_embed::RustEmbed;
use tao::event_loop::{ControlFlow, EventLoopBuilder};

/// Builder for the [`WintrayApp`].
///
/// Allows configuring the tray icon, tooltip, web router, and custom menu items.
pub struct WintrayAppBuilder {
    tooltip: String,
    icon_svg_bytes: Option<&'static [u8]>,
    router: Route,
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
            router: Route::new(),
            address: None,
            custom_menu_items: Vec::new(),
        }
    }

    /// Sets the tooltip text shown when hovering over the tray icon.
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = tooltip.into();
        self
    }

    /// Sets the SVG icon for the tray.
    /// The bytes should be a valid SVG string.
    pub fn with_icon(mut self, icon_svg_bytes: &'static [u8]) -> Self {
        self.icon_svg_bytes = Some(icon_svg_bytes);
        self
    }

    /// Configures the Poem endpoint for the embedded web UI.
    pub fn with_router<E>(mut self, router: E) -> Self
    where
        E: poem::IntoEndpoint + 'static,
        E::Endpoint: 'static,
    {
        self.router = self.router.nest("/", router);
        self
    }

    /// Automatically configures a Poem route to serve static assets from a `RustEmbed` implementation.
    pub fn with_assets<T: RustEmbed + Send + Sync + 'static>(
        mut self,
        path: impl Into<String>,
    ) -> Self {
        let path_str = path.into();
        let prefix = path_str.trim_end_matches('/');

        self.router = self.router.at(
            format!("{}/:path", prefix),
            get(make(move |req| async move {
                let path: Path<String> = Path::from_request_without_body(&req).await.unwrap();
                crate::assets::serve_embedded_assets::<T>(path).await
            })),
        );
        self
    }

    /// Sets the index page (/) to render the provided Askama template.
    /// The template must implement `Clone` and `Send + Sync + 'static`.
    pub fn with_index_template<T: Template + Clone + Send + Sync + 'static>(
        mut self,
        template: T,
    ) -> Self {
        self.router = self.router.at(
            "/",
            get(make(move |_| {
                let t = template.clone();
                async move { Html(t.render().unwrap()) }
            })),
        );
        self
    }

    /// Sets the address (e.g., "127.0.0.1:9876") the web server will bind to.
    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Adds a custom item to the tray context menu.
    pub fn add_menu_item(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.custom_menu_items.push((id.into(), label.into()));
        self
    }

    /// Builds the [`WintrayApp`] instance.
    ///
    /// # Panics
    /// Panics if the icon was not set using `.with_icon()`.
    pub fn build(self) -> WintrayApp {
        let address = self.address.unwrap_or_else(|| "127.0.0.1:9876".to_string());

        WintrayApp {
            tray_config: TrayConfig {
                tooltip: self.tooltip,
                icon_svg_bytes: self
                    .icon_svg_bytes
                    .expect("Icon must be set before building (use .with_icon())"),
                custom_menu_items: self.custom_menu_items,
            },
            router: self.router.boxed(),
            address,
        }
    }
}

/// The main application handle.
pub struct WintrayApp {
    tray_config: TrayConfig,
    router: BoxEndpoint<'static>,
    address: String,
}

impl WintrayApp {
    /// Starts the application and blocks the current thread.
    pub fn run(self) {
        self.run_with(|_| {});
    }

    /// Starts the application with a custom handler for tray menu events.
    ///
    /// The `custom_handler` closure is called with the ID of the clicked menu item.
    pub fn run_with<F>(self, mut custom_handler: F)
    where
        F: FnMut(&str) + 'static,
    {
        let address = self.address.clone();
        let ui_address = address.clone();

        // 1. Spawn the background web server with self-signed TLS support.
        let router = self.router;
        tokio::spawn(async move {
            let cert_path = std::path::Path::new("cert.pem");
            let key_path = std::path::Path::new("key.pem");

            if !cert_path.exists() || !key_path.exists() {
                println!("[Wintray] Generating self-signed certificate...");
                let cert = rcgen::generate_simple_self_signed(vec![
                    "localhost".to_string(),
                    "127.0.0.1".to_string(),
                ])
                .unwrap();
                std::fs::write(cert_path, cert.cert.pem()).unwrap();
                std::fs::write(key_path, cert.key_pair.serialize_pem()).unwrap();
            }

            let cert = std::fs::read(cert_path).unwrap();
            let key = std::fs::read(key_path).unwrap();
            let config = RustlsConfig::new()
                .fallback(poem::listener::RustlsCertificate::new().cert(cert).key(key));

            let listener = TcpListener::bind(address).rustls(config);
            Server::new(listener).run(router).await.unwrap();
        });

        // 2. Setup the Event Loop for tray interactions.
        let event_loop = EventLoopBuilder::<TrayUserEvent>::with_user_event().build();
        let proxy = event_loop.create_proxy();
        let _tray_icon = create_tray(proxy.clone(), self.tray_config);

        // 3. Start the application loop.
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            if let tao::event::Event::UserEvent(user_event) = event {
                match user_event {
                    TrayUserEvent::TrayIconEvent(tray_event) => {
                        // Handle left-click on the tray icon to open the web UI.
                        if let tray_icon::TrayIconEvent::Click {
                            button: tray_icon::MouseButton::Left,
                            button_state: tray_icon::MouseButtonState::Up,
                            ..
                        } = tray_event
                        {
                            let _ = open::that(format!("https://{}", ui_address));
                        }
                    }
                    TrayUserEvent::MenuEvent(menu_event) => {
                        if menu_event.id == "open" {
                            let _ = open::that(format!("https://{}", ui_address));
                        } else if menu_event.id == "close" {
                            *control_flow = ControlFlow::Exit;
                        } else {
                            // Forward other menu events to the custom handler.
                            custom_handler(menu_event.id.as_ref());
                        }
                    }
                }
            }
        });
    }
}
