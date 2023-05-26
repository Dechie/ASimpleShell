use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::env;
use std::process::Command;
use std::process::Child;
use std::process::Stdio;
use std::path::Path;

fn main() {
    
    // loop so that the process comes back again to take input/command.
    loop {
        let mut input = String::new();

        
        print!(">");
        stdout().flush(); 

        // trim the command so that it can take arguments
        // everything after the first whitespace character 
        // is interpreted as args to the command.
        stdin().read_line(&mut input).unwrap();

        // feature for enabling pipes: 
        // peek through string and trim at character |
        // command must be peekable so we know when we are on the last command.
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next() {
            // ^^read_line leaves a trailing new line, we remove this with trim
            let mut parts = input.trim().split_whitespace();
            
            let command = parts.next().unwrap(); 
            let args = parts;

            match command {
                "cd" => {
                    // default to '/' as new directory if directory isn't provided 
                    let new_dir = args.peekable().peek()
                        .map_or("/", |x| *x);
                    let root = Path::new(new_dir);

                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }

                    previous_command = None;
                },

                // so that it does not exit because of incorrect user input
                "exit" => return,
            
                command => {
                
                    let stdin = previous_command
                        .map_or(
                            Stdio::inherit(),
                            |output: Child| Stdio::from(output.stdout.unwrap())
                        );

                    let stdout = if commands.peek().is_some() {
                        // there is another command piped behind this one
                        // prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        // there is no command piped behind this one
                        // so send the output directly to shell stdout
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    // output is sort of a child process that enables taking input a second time,
                    // and also to enable error handling with simple outputs and with piped
                    // outputs.
                    match output {
                        Ok(output) => {previous_command = Some(output); },
                        Err(e) => {
                            previous_command = None;
                            eprintln!("{}", e);
                        },
                    };
            
                }
            }
            
        }

        // now we block input until the final command has finished.
        if let Some(mut final_command) = previous_command {
            final_command.wait();
        }

    }
}
