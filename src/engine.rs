use crate::tray::{TrayConfig, TrayUserEvent, create_tray};
use axum::Router;
use std::thread;
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};

pub struct ServiceEngine {
    pub tray_config: TrayConfig,
    pub router: Router,
    pub address: String,
}

impl ServiceEngine {
    pub fn new(tray_config: TrayConfig, router: Router, address: String) -> Self {
        Self { tray_config, router, address }
    }

    pub fn run<F>(self, mut event_handler: F)
    where
        F: FnMut(TrayUserEvent, &EventLoopProxy<TrayUserEvent>, &mut ControlFlow) + 'static,
    {
        let address = self.address.clone();

        // Используем роутер приложения напрямую
        let router = self.router;

        // 1. Запуск сервера
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
                event_handler(user_event, &proxy, control_flow);
            }
        });
    }
}
