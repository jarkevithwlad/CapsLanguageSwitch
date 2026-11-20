#![windows_subsystem = "windows"]
#![allow(non_snake_case)]
use std::{ffi::c_void, mem, ptr, thread, time::Duration};

type Handle = *mut c_void;
type LResult = isize;
#[repr(C)] struct Point { x: i32, y: i32 }
#[repr(C)] struct Msg { hwnd: Handle, message: u32, wParam: usize, lParam: isize, time: u32, pt: Point }
#[repr(C)] struct KeyboardLl { vkCode: u32, scanCode: u32, flags: u32, time: u32, dwExtraInfo: usize }
#[repr(C)] struct Input { kind: u32, data: InputData }
#[repr(C)] union InputData { ki: KeyInput, pad: [usize; 4] }
#[repr(C)] #[derive(Clone, Copy)] struct KeyInput { wVk: u16, wScan: u16, dwFlags: u32, time: u32, extra: usize }
const WH_KEYBOARD_LL:i32=13; const WM_KEYDOWN:u32=0x100; const WM_SYSKEYDOWN:u32=0x104; const WM_KEYUP:u32=0x101; const WM_SYSKEYUP:u32=0x105; const VK_CAPITAL:u32=0x14; const VK_LWIN:u16=0x5b; const VK_SPACE:u16=0x20; const KEYUP:u32=2;
#[link(name="user32")] extern "system" { fn SetWindowsHookExW(id:i32, cb:Option<unsafe extern "system" fn(i32,usize,isize)->LResult>, m:Handle, tid:u32)->Handle; fn CallNextHookEx(h:Handle, code:i32,w:usize,l:isize)->LResult; fn GetMessageW(m:*mut Msg,h:Handle,min:u32,max:u32)->i32; fn TranslateMessage(m:*const Msg)->i32; fn DispatchMessageW(m:*const Msg)->LResult; fn GetKeyState(key:i32)->i16; fn SendInput(n:u32,input:*const Input,size:i32)->u32; }
static mut DOWN: bool=false;
unsafe fn key(v:u16, up:bool)->Input { Input {kind:1,data:InputData{ki:KeyInput{wVk:v,wScan:0,dwFlags:if up{KEYUP}else{0},time:0,extra:0}}} }
unsafe extern "system" fn hook(code:i32,w:usize,l:isize)->LResult { if code>=0 { if (*(l as *const KeyboardLl)).flags & 0x10 != 0 { return CallNextHookEx(ptr::null_mut(),code,w,l); } let msg=w as u32; if msg==WM_KEYUP||msg==WM_SYSKEYUP { DOWN=false; if (*(l as *const KeyboardLl)).vkCode==VK_CAPITAL{return 1;} } if msg==WM_KEYDOWN||msg==WM_SYSKEYDOWN { if (*(l as *const KeyboardLl)).vkCode==VK_CAPITAL { if !DOWN { DOWN=true; println!("Caps Lock нажат — отправляю Win+Space"); let inputs=[key(VK_LWIN,false),key(VK_SPACE,false),key(VK_SPACE,true),key(VK_LWIN,true)]; let sent=SendInput(4,inputs.as_ptr(),mem::size_of::<Input>() as i32); println!("Win+Space: SendInput принял {sent}/4 событий"); thread::sleep(Duration::from_millis(150)); let state=GetKeyState(VK_CAPITAL as i32)&1!=0; println!("Состояние Caps Lock: {}",if state{"включён"}else{"выключен"}); if state { let caps=[key(VK_CAPITAL as u16,false),key(VK_CAPITAL as u16,true)]; let n=SendInput(2,caps.as_ptr(),mem::size_of::<Input>() as i32); println!("Отправлено выключение Caps Lock: {n}/2 событий"); } } return 1; } } } CallNextHookEx(ptr::null_mut(),code,w,l) }
fn main(){ unsafe { let h=SetWindowsHookExW(WH_KEYBOARD_LL,Some(hook),ptr::null_mut(),0); if h.is_null(){eprintln!("Не удалось установить клавиатурный перехватчик"); std::process::exit(1)} println!("Caps Lock переназначен на Win+Space. Для выхода нажмите Ctrl+C."); let mut m:Msg=mem::zeroed(); while GetMessageW(&mut m,ptr::null_mut(),0,0)>0 {TranslateMessage(&m);DispatchMessageW(&m);} } }
