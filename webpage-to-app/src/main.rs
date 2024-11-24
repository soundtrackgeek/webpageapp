use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use webview2_com::Microsoft::Web::WebView2::Win32;
use Win32::{
    CreateCoreWebView2EnvironmentWithOptions,
    ICoreWebView2Environment,
    ICoreWebView2Controller,
    ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler_Impl,
    ICoreWebView2CreateCoreWebView2ControllerCompletedHandler_Impl,
};
use windows::{
    core::{implement, Interface, Result as WindowsResult, HRESULT, PCWSTR, HSTRING},
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::System::Com::*,
    Win32::System::LibraryLoader::GetModuleHandleA,
};
use windows::core::s;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to load
    #[arg(short, long)]
    url: String,

    /// Window width
    #[arg(short, long, default_value = "800")]
    width: i32,

    /// Window height
    #[arg(short, long, default_value = "600")]
    height: i32,
}

#[implement(webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler)]
struct EnvironmentCompletedHandler {
    hwnd: HWND,
    url: String,
}

#[allow(non_snake_case)]
impl EnvironmentCompletedHandler {
    fn new(hwnd: HWND, url: String) -> Self {
        Self { hwnd, url }
    }
}

impl ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler_Impl for EnvironmentCompletedHandler {
    fn Invoke(&self, errorcode: HRESULT, environment: Option<&ICoreWebView2Environment>) -> WindowsResult<()> {
        let environment = environment.ok_or_else(|| windows::core::Error::from(errorcode))?;
        
        unsafe {
            environment.CreateCoreWebView2Controller(
                self.hwnd,
                &ControllerCompletedHandler::new(self.url.clone()),
            )
        }
    }
}

#[implement(webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2CreateCoreWebView2ControllerCompletedHandler)]
struct ControllerCompletedHandler {
    url: String,
}

#[allow(non_snake_case)]
impl ControllerCompletedHandler {
    fn new(url: String) -> Self {
        Self { url }
    }
}

impl ICoreWebView2CreateCoreWebView2ControllerCompletedHandler_Impl for ControllerCompletedHandler {
    fn Invoke(&self, errorcode: HRESULT, controller: Option<&ICoreWebView2Controller>) -> WindowsResult<()> {
        let controller = controller.ok_or_else(|| windows::core::Error::from(errorcode))?;
        
        unsafe {
            let webview = controller.CoreWebView2()?;
            webview.Navigate(PCWSTR::from_raw(self.url.encode_utf16().chain(Some(0)).collect::<Vec<_>>().as_ptr()))?;
        }
        Ok(())
    }
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
    unsafe { 
        CreateCoreWebView2EnvironmentWithOptions(
            None,
            None,
            None,
            &EnvironmentCompletedHandler::new(hwnd, args.url.to_string()),
        )?
    };

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
