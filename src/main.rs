// Задача
// Необходимо реализовать свою собственную UNIX-оболочку-утилиту с поддержкой ряда простейших команд:
//
// - cd <args> — смена директории (в качестве аргумента может быть то-то и то)
//
// - pwd — показать путь к текущему каталогу
//
// - echo <args> — вывод аргумента в STDOUT
//
// - kill <args> — процесс «убить», переданный в качестве аргумента
//
// - ps — выводит короткую информацию по запущенным процессам в формате id процесса, названия, времени работы в мсек.
//
// Так же требуется поддержка функциональной вилки/exec-команды
//
// Дополнительно необходимо поддерживать конвейер на пайпах (Linux Pipes, например cmd1 | cmd2 | .... | cmdN).


// *Shell — это обычная консольная программа, которая, будучи запущенной, в интерактивном сеансе выводит некое приглашение в STDOUT и ожидает ввода пользователя через STDIN.
// Дождавшись ввода, обрабатывает команду согласно своей логике и при необходимости выводит результат на экран.
// Интерактивный сеанс выполняется до тех пор, пока не будет введена команда вывода (например, \quit).


use std::env; // Для работы с окружением
use std::io::{self, Write, BufRead}; // Для ввода/вывода
use std::process::{Command, Stdio}; // Для выполнения команд

fn main() {
    let stdin = io::stdin(); // Получаем стандартный ввод
    let mut reader = stdin.lock(); // Блокируем ввод для чтения построчно

    loop {
        print!("myshell> "); // Печатаем приглашение
        io::stdout().flush().unwrap(); // Обеспечиваем вывод на экран

        let mut input = String::new(); // Строка для хранения ввода
        if reader.read_line(&mut input).is_err() {
            break; // Если произошла ошибка чтения, выходим
        }
        let input = input.trim(); // Удаляем пробелы в начале и конце

        if input == "quit" {
            break; // Выход из оболочки
        }

        // Обработка конвейеров (pipeline)
        let commands: Vec<&str> = input.split('|').map(|s| s.trim()).collect(); // Разделяем команды
        if commands.len() > 1 {
            execute_pipeline(&commands); // Если есть несколько команд, выполняем конвейер
        } else {
            let args: Vec<&str> = input.split_whitespace().collect(); // Разделяем аргументы
            match args[0] {
                "cd" => { // Команда смены директории
                    if let Err(e) = env::set_current_dir(args.get(1).unwrap_or(&"~").to_string()) {
                        eprintln!("cd failed: {}", e); // Выводим ошибку, если не удалось сменить директорию
                    }
                }
                "pwd" => { // Команда вывода текущей директории
                    if let Ok(cwd) = env::current_dir() {
                        println!("{}", cwd.display()); // Печатаем текущую директорию
                    }
                }
                "echo" => { // Команда вывода текста
                    println!("{}", args[1..].join(" ")); // Печатаем все аргументы после echo
                }
                "kill" => { // Команда завершения процесса
                    if let Ok(pid) = args[1].parse::<i32>() {
                        let _ = Command::new("kill").arg(pid.to_string()).output(); // Убиваем процесс с указанным PID
                    }
                }
                "ps" => list_processes(), // Команда для вывода процессов
                _ => execute_command(&args), // Выполняем любую другую команду
            }
        }
    }
}

fn execute_pipeline(commands: &[&str]) {
    let mut processes: Vec<std::process::Child> = Vec::new(); // Храним дочерние процессы

    for (i, command) in commands.iter().enumerate() {
        let parts: Vec<&str> = command.split_whitespace().collect(); // Разделяем команду на части

        // Запускаем команду с соответствующими параметрами
        let mut child = Command::new(parts[0])
            .args(&parts[1..])
            .stdin(if i > 0 { Stdio::piped() } else { Stdio::inherit() }) // Если не первая команда, получаем ввод из предыдущей
            .stdout(if i < commands.len() - 1 { Stdio::piped() } else { Stdio::inherit() }) // Если не последняя команда, выводим в следующую
            .spawn()
            .expect("Failed to start command"); // Запускаем команду

        // Если не первая команда, перенаправляем stdout предыдущей команды в stdin текущей
        if i > 0 {
            let previous_stdout = processes[i - 1].stdout.take().expect("Failed to take stdout"); // Получаем stdout предыдущей команды
            child.stdout = Some(previous_stdout); // Устанавливаем его как stdin текущей команды
        }

        processes.push(child); // Сохраняем дочерний процесс
    }

    // Ждем завершения всех процессов
    for mut process in processes {
        let _ = process.wait().expect("Failed to wait for command"); // Ждем завершения
    }
}

fn execute_command(args: &[&str]) {
    match args[0] {
        "dir" => {
            // Выполняем dir с помощью командной строки
            let output = Command::new("cmd")
                .args(&["/C", "dir"]) // Выполняем команду dir
                .output()
                .expect("Failed to execute dir command");
            // Выводим результат
            println!("{}", String::from_utf8_lossy(&output.stdout)); // Печатаем вывод
        }
        _ => {
            let status = Command::new(args[0])
                .args(&args[1..]) // Запускаем любую другую команду
                .status()
                .expect("Command failed to execute");

            if !status.success() {
                eprintln!("Command exited with status: {}", status); // Выводим статус завершения команды
            }
        }
    }
}

fn list_processes() {
    #[cfg(target_os = "linux")]
    {
        // Код для Linux (не реализован)
    }

    #[cfg(target_os = "windows")]
    {
        use sysinfo::{System}; // Импортируем библиотеку для получения информации о системе

        let mut system = System::new(); // Создаем новый объект системы
        system.refresh_all(); // Обновляем информацию о процессах

        for (pid, process) in system.processes() { // Проходим по всем процессам
            let name = process.name().to_string_lossy(); // Преобразование имени процесса в строку
            let cpu_usage = process.cpu_usage(); // Получаем использование CPU
            println!("{} {} {:.2}%", pid, name, cpu_usage); // Печатаем PID, имя и использование CPU
        }
    }
}
