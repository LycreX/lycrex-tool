use std::io::{self, Write};

static mut UNIQUE_HEALTH: i32 = 12345;
static mut UNIQUE_MANA: i32 = 54321;  
static mut UNIQUE_COINS: i32 = 98765;
static mut UNIQUE_LEVEL: i32 = 13579;

fn main() {
    println!("PID: {}", std::process::id());
    
    let mut heap_gems = Box::new(24680i32);
    let mut heap_exp = Box::new(86420i32);
    let mut heap_score = Box::new(11223344i32);
    
    let mut stack_ammo = 77777i32;
    let mut stack_energy = 88888i32;
    
    print_all_values(&heap_gems, &heap_exp, &heap_score, stack_ammo, stack_energy);
    interactive_loop(&mut heap_gems, &mut heap_exp, &mut heap_score, &mut stack_ammo, &mut stack_energy);
}

fn print_all_values(heap_gems: &Box<i32>, heap_exp: &Box<i32>, heap_score: &Box<i32>, stack_ammo: i32, stack_energy: i32) {
    println!("Global:");
    println!("  Health: {} (Addr: 0x{:X})", unsafe { UNIQUE_HEALTH }, std::ptr::addr_of!(UNIQUE_HEALTH) as usize );
    println!("  Mana:   {} (Addr: 0x{:X})", unsafe { UNIQUE_MANA }, std::ptr::addr_of!(UNIQUE_MANA) as usize );
    println!("  Coins:  {} (Addr: 0x{:X})", unsafe { UNIQUE_COINS }, std::ptr::addr_of!(UNIQUE_COINS) as usize );
    println!("  Level:  {} (Addr: 0x{:X})", unsafe { UNIQUE_LEVEL }, std::ptr::addr_of!(UNIQUE_LEVEL) as usize);
    
    println!("Heap:");
    println!("  Gems:   {} (Addr: {:p})", **heap_gems, heap_gems.as_ref());
    println!("  Exp:    {} (Addr: {:p})", **heap_exp, heap_exp.as_ref());
    println!("  Score:  {} (Addr: {:p})", **heap_score, heap_score.as_ref());
    
    println!("Stack:");
    println!("  Ammo:   {} (Addr: {:p})", stack_ammo, &stack_ammo);
    println!("  Energy: {} (Addr: {:p})", stack_energy, &stack_energy);
}

fn interactive_loop(heap_gems: &mut Box<i32>, heap_exp: &mut Box<i32>, heap_score: &mut Box<i32>, stack_ammo: &mut i32, stack_energy: &mut i32) {
    loop {
        print!("\nCommand: (h)ealth, (m)ana, (c)oins, (l)evel, (g)ems, (e)xp, (s)core, (a)mmo, en(r)gy, (v)iew, (q)uit: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        
        match input.trim().to_lowercase().as_str() {
            "h" | "health" => {
                unsafe { UNIQUE_HEALTH += 1111; }
                println!("Health increased to: {}", unsafe { UNIQUE_HEALTH });
            },
            "m" | "mana" => {
                unsafe { UNIQUE_MANA += 2222; }
                println!("Mana increased to: {}", unsafe { UNIQUE_MANA });
            },
            "c" | "coins" => {
                unsafe { UNIQUE_COINS += 3333; }
                println!("Coins increased to: {}", unsafe { UNIQUE_COINS });
            },
            "l" | "level" => {
                unsafe { UNIQUE_LEVEL += 1; }
                println!("Level increased to: {}", unsafe { UNIQUE_LEVEL });
            },
            "g" | "gems" => {
                **heap_gems += 5555;
                println!("Gems increased to: {}", **heap_gems);
            },
            "e" | "exp" => {
                **heap_exp += 6666;
                println!("Exp increased to: {}", **heap_exp);
            },
            "s" | "score" => {
                **heap_score += 7777;
                println!("Score increased to: {}", **heap_score);
            },
            "a" | "ammo" => {
                *stack_ammo += 999;
                println!("Ammo increased to: {}", *stack_ammo);
            },
            "r" | "nrg" | "energy" => {
                *stack_energy += 1234;
                println!("Energy increased to: {}", *stack_energy);
            },
            "v" | "view" => {
                print_all_values(heap_gems, heap_exp, heap_score, *stack_ammo, *stack_energy);
            },
            "q" | "quit" => {
                println!("Bye!");
                break;
            },
            "test" => {
                println!("Starting auto test sequence...");
                for i in 1..=5 {
                    unsafe { 
                        UNIQUE_HEALTH = 12345 + i * 1000;
                        UNIQUE_MANA = 54321 + i * 1000;
                    }
                    **heap_gems = 24680 + i * 1000;
                    println!("Test {}: Health={}, Mana={}, Gems={}", 
                        i, unsafe { UNIQUE_HEALTH }, unsafe { UNIQUE_MANA }, **heap_gems);
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            },
            "" => continue,
            _ => {
                println!("Unknown command. Available commands: h, m, c, l, g, e, s, a, r, v, q, test");
            }
        }
    }
}
