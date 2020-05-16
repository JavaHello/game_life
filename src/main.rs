#[macro_use]
extern crate lazy_static;
#[cfg(windows)]
extern crate winapi;

use std::ffi::OsStr;
use std::fmt;
use std::io::Error;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::sync::RwLock;

use rand;
use rand::Rng;
#[cfg(windows)]
use winapi::_core::ptr::null_mut;
// #[cfg(windows)]
// use winapi::ctypes::*;
#[cfg(windows)]
use winapi::shared::basetsd::*;
#[cfg(windows)]
use winapi::shared::minwindef::*;
#[cfg(windows)]
use winapi::shared::windef::*;
#[cfg(windows)]
use winapi::um::libloaderapi::*;
#[cfg(windows)]
use winapi::um::wingdi::*;
#[cfg(windows)]
use winapi::um::winuser::*;

const _WIDTH: i32 = 800;
const _HEIGHT: i32 = 800;
const CELL_SIZE: i32 = 64;
const COL_LEN: i32 = _WIDTH / CELL_SIZE as i32;
const ROW_LEN: i32 = _HEIGHT / CELL_SIZE as i32;

const WIDTH: i32 = COL_LEN * CELL_SIZE + COL_LEN * 7;
const HEIGHT: i32 = ROW_LEN * CELL_SIZE + ROW_LEN * 9;

lazy_static! {
 static  ref   UNIVERSE:RwLock<Universe> = RwLock::new(Universe::new());
}

#[derive(Copy, PartialEq, Clone)]
enum Cell {
    Alive = 1,
    Dead = 0,
}

pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    count: i64,
}

impl Universe {
    pub fn new() -> Universe {
        let mut rag = rand::thread_rng();
        let width = CELL_SIZE as u32;
        let height = CELL_SIZE as u32;

        let cells = (0..width * height)
            .map(|_| {
                let r: i32 = rag.gen_range(1, 10);
                if r > 5 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();
        Universe {
            width,
            height,
            cells,
            count: 0,
        }
    }
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn set_cell(&mut self, row: u32, column: u32) {
        let index = self.get_index(row, column);
        self.cells.remove(index);
        self.cells.insert(index, Cell::Alive);
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }
                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8
            }
        }

        count
    }
}

impl Universe {
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);
                let next_cell = match (cell, live_neighbors) {
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (otherwise, _) => otherwise,
                };
                next[idx] = next_cell;
            }
        }
        self.cells = next;
        self.count += 1;
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[cfg(windows)]
fn print_message(msg: &str) -> Result<i32, Error> {
    let wide: Vec<u16> = OsStr::new(msg).encode_wide().chain(once(0)).collect();
    let ret = unsafe { MessageBoxW(null_mut(), wide.as_ptr(), wide.as_ptr(), MB_OK) };
    if ret == 0 {
        Err(Error::last_os_error())
    } else {
        Ok(ret)
    }
}

#[cfg(not(windows))]
fn print_message(msg: &str) -> Result<(), Error> {
    println!("{}", msg);
    Ok(())
}

fn draw_rec(cell: &Cell, hdc: HDC, c: i32, r: i32) {
    unsafe {
        let hbr = match cell {
            Cell::Alive => {
                CreateSolidBrush(RGB(0, 0, 0))
            }
            Cell::Dead => {
                CreateSolidBrush(RGB(255, 255, 255))
            }
        };
        let rec = RECT {
            left: c * (COL_LEN + 1) + 1,
            top: r * (ROW_LEN + 1) + 1,
            right: c * (COL_LEN + 1) + COL_LEN,
            bottom: r * (ROW_LEN + 1) + ROW_LEN,
        };
        // 画刷选择到当前DC中
        let org_brs = SelectObject(hdc, hbr as HGDIOBJ) as HBRUSH;
        // Rectangle(hdc, c * (COL_LEN + 1) + 1, r * (ROW_LEN + 1) + 1, c * (COL_LEN + 1) + COL_LEN, r * (ROW_LEN + 1) + ROW_LEN);

        FillRect(
            hdc,
            &rec,
            hbr,
        );

        // 选回原先的画刷
        SelectObject(hdc, org_brs as HGDIOBJ);
        DeleteObject(hbr as HGDIOBJ);
    }
}

#[cfg(windows)]
unsafe extern "system" fn window_proc(hwnd: HWND, u_msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match u_msg {
        WM_CLOSE => {
            DestroyWindow(hwnd);
        }
        WM_DESTROY => {
            PostQuitMessage(u_msg as i32);
        }
        WM_CREATE => {
            SetTimer(hwnd, 0, 10, Some(tick_run));
            // SetTimer(hwnd, 1, 10, Some(draw_run));
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = PAINTSTRUCT {
                hdc: null_mut(),
                fErase: 0,
                rcPaint: RECT {
                    left: 0,
                    right: 0,
                    bottom: 0,
                    top: 0,
                },
                fRestore: 0,
                fIncUpdate: 0,
                rgbReserved: [0; 32],
            };
            let hdc = BeginPaint(hwnd, &mut ps);
            for i in 0..=CELL_SIZE {
                MoveToEx(hdc, 0, i * (ROW_LEN + 1), null_mut());
                LineTo(hdc, (ROW_LEN + 1) * CELL_SIZE, i * (ROW_LEN + 1));

                MoveToEx(hdc, i * (COL_LEN + 1), 0, null_mut());
                LineTo(hdc, i * (COL_LEN + 1), (COL_LEN + 1) * CELL_SIZE);
            }
            EndPaint(hwnd, &ps);
        }
        WM_LBUTTONDOWN => {
            let hdc = GetDC(hwnd);
            let x_pos = LOWORD(l_param as u32);
            let y_pos = HIWORD(l_param as u32);
            let col = x_pos / (COL_LEN + 1) as u16;
            let row = y_pos / (ROW_LEN + 1) as u16;
            let mut u = UNIVERSE.write().unwrap();
            u.set_cell(col as u32, row as u32);
            draw_rec(&Cell::Alive, hdc, col as i32, row as i32);
            // SendMessageW(hwnd, WM_DRAWITEM, 0, 0);
            ReleaseDC(hwnd, hdc);   //归还系统绘图设备
        }
        WM_DRAWITEM => {
            let hdc = GetDC(hwnd);
            let u = UNIVERSE.read().unwrap();
            // println!("{}", u);
            let z = format!("周期: {}", u.count).encode_utf16().collect::<Vec<u16>>();
            // SetWindowTextW(hwnd, z.as_ptr());
            TextOutW(hdc, CELL_SIZE * (COL_LEN + 0) - 2 * COL_LEN, 0, z.as_ptr(), z.len() as i32);
            for c in 0..CELL_SIZE {
                for r in 0..CELL_SIZE {
                    draw_rec(&u.cells[u.get_index(c as u32, r as u32)], hdc, c, r);
                }
            }

            // BitBlt(hdc, 0, 0, WIDTH, HEIGHT, mem_dc, 0, 0, SRCCOPY);//复制到系统设备上显示
            // DeleteDC(mem_dc);        //释放辅助绘图设备
            ReleaseDC(hwnd, hdc);   //归还系统绘图设备
        }
        _ => ()
    };
    return DefWindowProcA(hwnd, u_msg, w_param, l_param);
}

#[cfg(windows)]
unsafe extern "system" fn tick_run(
    hwnd: HWND,
    _a: UINT,
    _b: UINT_PTR,
    _d: DWORD,
) {
    UNIVERSE.write().unwrap().tick();
    SendMessageW(hwnd, WM_DRAWITEM, 0, 0);
}

#[cfg(windows)]
unsafe extern "system" fn draw_run(
    hwnd: HWND,
    _a: UINT,
    _b: UINT_PTR,
    _d: DWORD,
) {
    SendMessageW(hwnd, WM_DRAWITEM, 0, 0);
}

#[cfg(windows)]
fn create_windows(title: &str) -> Result<(), Error> {

    // let wide: Vec<u16> = title.to_string().encode_utf16().chain(once(0)).collect();
    let wide: Vec<u16> = OsStr::new(title).encode_wide().collect();
    unsafe {
        let h_instance: HINSTANCE = GetModuleHandleW(null_mut());
        let wnd_class = WNDCLASSEXW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: LoadIconW(null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(null_mut(), IDC_HAND),
            hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
            lpszMenuName: null_mut(),
            lpszClassName: wide.as_ptr(),
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            hIconSm: LoadIconW(null_mut(), IDI_APPLICATION),
        };
        RegisterClassExW(&wnd_class);
        let hwnd = CreateWindowExW(
            WS_EX_APPWINDOW,
            wnd_class.lpszClassName,
            wide.as_ptr(),
            WS_EX_LAYERED | WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            WIDTH,
            HEIGHT,
            null_mut(),
            null_mut(),
            h_instance,
            null_mut(),
        );
        ShowWindow(hwnd, SW_SHOWNORMAL);
        let mut msg = MSG {
            hwnd: null_mut(),
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: POINT {
                x: 0,
                y: 0,
            },
        };
        while GetMessageW(&mut msg, null_mut(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}


fn main() {
    create_windows("生命游戏").unwrap();
}