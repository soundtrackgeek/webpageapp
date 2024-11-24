use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use webview2_com::{Microsoft::Web::WebView2::Win32::*, WebView2Environment};
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::System::Com::*,
    Win32::System::LibraryLoader::GetModuleHandleA,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL of the webpage to convert
    #[arg(short, long)]
    url: String,

    /// Output directory for the generated app
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Window width
    #[arg(short, long, default_value = "800")]
    width: i32,

    /// Window height
    #[arg(short, long, default_value = "600")]
    height: i32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize COM
    unsafe { CoInitializeEx(
        None,
        COINIT_APARTMENTTHREADED,
    )?; }

    // Create window class
    let instance = unsafe { GetModuleHandleA(None)? };
    let window_class = s!("WebpageApp");
    
    let wc = WNDCLASSA {
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
        hInstance: instance.into(),
        lpszClassName: window_class,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        ..Default::default()
    };

    unsafe { RegisterClassA(&wc) };

    // Create window
    let hwnd = unsafe {
        CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            window_class,
            s!("Webpage App"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            args.width,
            args.height,
            None,
            None,
            instance,
            None,
        )
    };

    // Create WebView2 environment
    let environment = WebView2Environment::builder().build()?;
    let controller = environment.create_controller(hwnd, |_| Ok(()))?;
    let webview = controller.get_webview()?;

    // Navigate to URL
    webview.navigate(&args.url)?;

    // Message loop
    let mut message = MSG::default();
    
    unsafe {
        while GetMessageA(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageA(&message);
        }
    }

    Ok(())
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(hwnd, msg, wparam, lparam),
        }
    }
}
