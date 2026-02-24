fn main() {
	slint_build::compile("ui/app.slint").unwrap();
	#[cfg(target_os = "windows")]
	{
		let mut res = winresource::WindowsResource::new();
		res.set_icon("res/icon.ico");
		res.compile().unwrap();
	}
}