use log::info;
use tauri::{
    menu::{IsMenuItem, MenuBuilder, MenuItemBuilder},
    tray::{TrayIcon, TrayIconBuilder},
    Manager,
};

pub fn create_system_tray(app: &tauri::App) -> Result<TrayIcon, tauri::Error> {
    // 创建菜单项
    let exit_item = MenuItemBuilder::new("退出")
        .id("exit")
        .build(app.handle())?;

    // 将菜单项包装为对 dyn IsMenuItem 的引用
    let items: [&dyn IsMenuItem<_>; 1] = [&exit_item];

    // 使用 MenuBuilder 创建菜单并添加菜单项
    let menu = MenuBuilder::new(app.handle()).items(&items).build()?;

    // 构建系统托盘
    TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("MemeMeow 表情包助手")
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(|app, event| {
            if event.id() == "exit" {
                info!("用户通过系统托盘菜单退出程序");
                app.exit(0);
            }
        })
        .build(app)
}
