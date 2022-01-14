// Module to hold information about the application, like help and info

const HELP: &'static [(&'static str, &'static str)] = &[
    (&"-v", &"version information"),
    (&"version", &"version information"),
    (&"-h", &"usage information"),
    (&"help", &"usage information"),
    (&"add", &"interactively add a reminder")
];

pub fn help_info(){
    println!(
	"Caduceus, a client for Hermes\n\
	 Usage: caduceus [OPTION]\n\n\
	 Options:"
    );

    for (opt,long) in HELP.iter() {
	println!("\t{}\t\t{}",opt,long);
    }
}

pub fn version_info(){
    println!("Caduceus version 0.1.0");
}
