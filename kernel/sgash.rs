/* kernel::sgash.rs */
#[allow(unused_imports)];
use core::*;
use core::str::*;
use core::option::{Some, Option, None};
use core::iter::Iterator;
use kernel::*;
use super::super::platform::*;
use kernel::memory::Allocator;

// Used to store character into QEMU
pub fn putchar(key: char) {
    unsafe {
        /*
        * We need to include a blank asm call to prevent rustc
        * from optimizing this part out
        */
        asm!("");
        io::write_char(key, io::UART0);
    }
}

fn putstr(msg: &str) {
    for c in slice::iter(as_bytes(msg)) {
        putchar(*c as char);
    }   
}

pub unsafe fn drawstr(msg: &str) {
    // See starting code to change color of print out
    for c in slice::iter(as_bytes(msg)) {
        drawchar(*c as char);
    }
}

// Draws character onto QEMU screen
unsafe fn drawchar(x: char)
{
    io::restore();
    if x == '\n' {
    io::CURSOR_Y += io::CURSOR_HEIGHT;
        io::CURSOR_X = 0u32;
        return;
    } else {
io::draw_char(x);   
io::CURSOR_X += io::CURSOR_WIDTH;
    }
    io::backup();
    io::draw_cursor();
}

unsafe fn backspace()
{
    io::restore();
    if (io::CURSOR_X >= io::CURSOR_WIDTH) {
io::CURSOR_X -= io::CURSOR_WIDTH;
io::draw_char(' ');
    }
    io::backup();
    io::draw_cursor();
}


// Used to get keystroke and print onscreen
pub unsafe fn parsekey(x: char) {
    let x = x as u8;
    match x {
        13  =>  {
            parse();
            prompt(false);
        }
        127 =>  {
            if(buffer.delete_char()) {
                putchar('');
                putchar(' ');
                putchar('');
                backspace();
            }
        }
        _   =>  {
            if (buffer.add_char(x)) {
                putchar(x as char);
                drawchar(x as char);
            }
        }
    }
}



pub unsafe fn init() {
    buffer = cstr::new(256);
    screen();
    // First prompt does draw, but out of bounds?
    prompt(true);
}

/****************** Added Code ***************/
pub static mut buffer: cstr = cstr {
    pointer: 0 as *mut u8,
    index_pointer: 0,
    max: 0
};

// Problem 1: Prompt on each line
unsafe fn prompt(startup: bool) {
    putstr(&"\nsgash > ");
    if !startup {
        drawstr(&"\nsgash > ");
    } else {
        // Technically it's there, but it's not visible
        // Unlikely color issue, maybe something with CRUSOR?
        drawstr(&"\nstart >");
    }
    buffer.reset();
}

// Problem 4: echo input on second line
unsafe fn echo() {
    putstr(&"\n");
    drawstr(&"\n");
    let mut i: uint = 0;
    while i < buffer.len(){
        let current_char: char = buffer.get_char(i);

        // Occassionally gets wrong char, probably something to do with the cstr implementation
        putchar(current_char);
        drawchar(current_char);       // Hangs without the first sgash> prompt
        i += 1;
    }
    buffer.reset();
}


// Problem 5: Recognize shell commands
unsafe fn parse() {
    if (buffer.streq(&"ls")) { 
        putstr( &"\na\tb") ;
        drawstr( &"\na    b") ;
        buffer.reset();
    }
    else {
        putstr(&"\nparsing command...");
        match buffer.getarg(' ', 0){
            // Hangs here
            Some(cmd)   => {
                putstr(&"\nmatching...");
                drawstr(&"\nmatching...");
            },
            None        => {
                putstr(&"\nDidn't recognize command...");
                drawstr(&"\nDidn't recognize command...");
            }
        }

        buffer.reset();
    }

    /*
    if (buffer.streq(&"echo")) { 
        putstr( &"\necho");
        drawstr( &"\necho");
        buffer.reset();
    };
    // Separates what's in buffer by spliting spaces
    match buffer.getarg(' ', 0) {
        Some(cmd)        => {
            if(cmd.streq(&"cat")) {
                match buffer.getarg(' ', 1) {
                Some(x)        => {
                    if(x.streq(&"a")) { 
                    putstr( &"\nHowdy!"); 
                    drawstr( &"\nHowdy!"); 
                    }
                    if(x.streq(&"b")) {
                    putstr( &"\nworld!");
                    drawstr( &"\nworld!");
                    }
                }
                None        => { }
                };
            }
            
            if cmd.streq(&"echo") {
                putstr(&"\necho");
                drawstr(&"\necho");
            }
        }
        None        => {}
    };
    */
    
}


/* CString */
struct cstr {
    pointer: *mut u8,       // Starting address
    index_pointer: uint,    // Address of i
    max: uint               // Maximum end address           
}

impl cstr {
    pub unsafe fn new(size: uint) -> cstr {
        // Sometimes this doesn't allocate enough memory and gets stuck...
        let (addr_offset,mem_size) = heap.alloc(size);
        let this = cstr {
            pointer:            addr_offset,
            index_pointer:      0,
            max:                mem_size
        };
        // String requires null terminator
        *(((this.pointer as uint)+this.index_pointer) as *mut char) = '\0';
        this
    }

#[allow(dead_code)]
    unsafe fn from_str(s: &str) -> cstr {
        let mut this = cstr::new(256);
        for c in slice::iter(as_bytes(s)) {
            this.add_char(*c);
        };
        this
    }

#[allow(dead_code)]
    fn len(&self) -> uint { self.index_pointer }

    // HELP THIS DOESN'T WORK THERE IS NO GARBAGE COLLECTION!!!
    // -- TODO: exchange_malloc, exchange_free
#[allow(dead_code)]
    unsafe fn destroy(&self) { heap.free(self.pointer); }

    unsafe fn add_char(&mut self, x: u8) -> bool{
        if (self.index_pointer == self.max) { return false; }
        *(((self.pointer as uint)+self.index_pointer) as *mut u8) = x;
        self.index_pointer += 1;
        *(((self.pointer as uint)+self.index_pointer) as *mut char) = '\0';
        true
    }

    // Used for echo function
    unsafe fn get_char(&mut self, index: uint) -> char {
        if self.index_pointer == 0 {
            return (0 as u8) as char;
        }
        return *(((self.pointer as uint)+index) as *mut char);
    }

    unsafe fn delete_char(&mut self) -> bool {
        if (self.index_pointer == 0) { return false; }
        self.index_pointer -= 1;
        *(((self.pointer as uint)+self.index_pointer) as *mut char) = '\0';
        true
    }

    unsafe fn reset(&mut self) {
        let length = self.len();
        let mut i = length-1;
        while i < length {
            self.delete_char();
            i -= 1;
        }
        self.index_pointer = 0; 
        *(self.pointer as *mut char) = '\0';
    }

#[allow(dead_code)]
    unsafe fn eq(&self, other: &cstr) -> bool {
        if (self.len() != other.len()) { return false; }
        else {
            let mut x = 0;
            let mut selfp: uint = self.pointer as uint;
            let mut otherp: uint = other.pointer as uint;
            while x < self.len() {
                if (*(selfp as *char) != *(otherp as *char)) { return false; }
                selfp += 1;
                otherp += 1;
                x += 1;
            }
            true
        }
    }

    unsafe fn streq(&self, other: &str) -> bool {
        let mut selfp: uint = self.pointer as uint;
        for c in slice::iter(as_bytes(other)) {
            if( *c != *(selfp as *u8) ) { return false; }
            selfp += 1;
        };
        *(selfp as *char) == '\0'
    }

    unsafe fn getarg(&self, delim: char, mut k: uint) -> Option<cstr> {
        let mut ind: uint = 0;
        let mut found = k == 0;
        let mut selfp: uint = self.pointer as uint;
        let mut s = cstr::new(256);
        loop {
            if (*(selfp as *char) == '\0') { 
                // End of string
                if (found) { return Some(s); }
                else { return None; }
            };
            if (*(selfp as *u8) == delim as u8) { 
                if (found) { return Some(s); }
                k -= 1;
            };
            if (found) {
                s.add_char(*(selfp as *u8));
            };
            found = k == 0;
            selfp += 1;
            ind += 1;
            if (ind == self.max) { 
                putstr(&"\nSomething broke!");
                return None; 
            }
        }
    }

#[allow(dead_code)]
    unsafe fn split(&self, delim: char) -> (cstr, cstr) {
        let mut selfp: uint = self.pointer as uint;
        let mut beg = cstr::new(256);
        let mut end = cstr::new(256);
        let mut found = false;
        loop {
            if (*(selfp as *char) == '\0') { 
                return (beg, end);
            }
            else if (*(selfp as *u8) == delim as u8) {
                found = true;
            }
            else if (!found) {
                beg.add_char(*(selfp as *u8));
            }
            else if (found) {
                end.add_char(*(selfp as *u8));
            };
            selfp += 1;
        }
    }

    unsafe fn get_cmd(&self, delim: char, mut k: uint) -> cstr {
        let mut ind: uint = 0;
        let mut found = k == 0;
        let mut selfp: uint = self.pointer as uint;
        let mut s = cstr::new(256);
        loop {
            if (*(selfp as *char) == '\0') { 
                // End of string
                if (found) { return s; }
                else { s.reset(); return s;  }
            };
            if (*(selfp as *u8) == delim as u8) { 
                if (found) { return s; }
                k -= 1;
            };
            if (found) {
                s.add_char(*(selfp as *u8));
            };
            found = k == 0;
            selfp += 1;
            ind += 1;
            if (ind == self.max) { 
                putstr(&"\nSomething broke!");
                s.reset();
                return s; 
            }
        }
    }

}





// No real purpose beside printing on bash
fn screen() {
    putstr(&"\n ");
    putstr(&"\n ");
    putstr(&"\n 7=..~$=..:7 ");
    putstr(&"\n +$: =$$$+$$$?$$$+ ,7? ");
    putstr(&"\n $$$$$$$$$$$$$$$$$$Z$$ ");
    putstr(&"\n 7$$$$$$$$$$$$. .Z$$$$$Z$$$$$$ ");
    putstr(&"\n ~..7$$Z$$$$$7+7$+.?Z7=7$$Z$$Z$$$..: ");
    putstr(&"\n ~$$$$$$$$7: :ZZZ, :7ZZZZ$$$$= ");
    putstr(&"\n Z$$$$$? .+ZZZZ$$ ");
    putstr(&"\n +$ZZ$$$Z7 7ZZZ$Z$$I. ");
    putstr(&"\n $$$$ZZZZZZZZZZZZZZZZZZZZZZZZI, ,ZZZ$$Z ");
    putstr(&"\n :+$$$$ZZZZZZZZZZZZZZZZZZZZZZZZZZZ= $ZZ$$+~, ");
    putstr(&"\n ?$Z$$$$ZZZZZZZZZZZZZZZZZZZZZZZZZZZZI 7ZZZ$ZZI ");
    putstr(&"\n =Z$$+7Z$$7ZZZZZZZZ$$$$$$$ZZZZZZZZZZ ~Z$?$ZZ? ");    
    putstr(&"\n :$Z$Z...$Z $ZZZZZZZ~ ~ZZZZZZZZ,.ZZ...Z$Z$~ ");
    putstr(&"\n 7ZZZZZI$ZZ $ZZZZZZZ~ =ZZZZZZZ7..ZZ$?$ZZZZ$ ");
    putstr(&"\n ZZZZ$: $ZZZZZZZZZZZZZZZZZZZZZZ= ~$ZZZ$: ");
    putstr(&"\n 7Z$ZZ$, $ZZZZZZZZZZZZZZZZZZZZ7 ZZZ$Z$ ");
    putstr(&"\n =ZZZZZZ, $ZZZZZZZZZZZZZZZZZZZZZZ, ZZZ$ZZ+ ");
    putstr(&"\n ,ZZZZ, $ZZZZZZZ: =ZZZZZZZZZ ZZZZZ$: ");
    putstr(&"\n =$ZZZZ+ ZZZZZZZZ~ ZZZZZZZZ~ =ZZZZZZZI ");
    putstr(&"\n $ZZ$ZZZ$$Z$$ZZZZZZZZZ$$$$ IZZZZZZZZZ$ZZZZZZZZZ$ ");
    putstr(&"\n :ZZZZZZZZZZZZZZZZZZZZZZ ~ZZZZZZZZZZZZZZZZZ~ ");
    putstr(&"\n ,Z$$ZZZZZZZZZZZZZZZZZZZZ ZZZZZZZZZZZZZZZZZZ~ ");
    putstr(&"\n =$ZZZZZZZZZZZZZZZZZZZZZZ $ZZZZZZZZZZZZZZZ$+ ");
    putstr(&"\n IZZZZZ:. . ,ZZZZZ$ ");
    putstr(&"\n ~$ZZZZZZZZZZZ ZZZZ$ZZZZZZZ+ ");
    putstr(&"\n Z$ZZZ. ,Z~ =Z:.,ZZZ$Z ");
    putstr(&"\n ,ZZZZZ..~Z$. .7Z:..ZZZZZ: ");
    putstr(&"\n ~7+:$ZZZZZZZZI=:. .,=IZZZZZZZ$Z:=7= ");
    putstr(&"\n $$ZZZZZZZZZZZZZZZZZZZZZZ$ZZZZ ");
    putstr(&"\n ==..$ZZZ$ZZZZZZZZZZZ$ZZZZ .~+ ");
    putstr(&"\n I$?.?ZZZ$ZZZ$ZZZI =$7 ");
    putstr(&"\n $7..I$7..I$, ");
    putstr(&"\n");
    putstr(&"\n _ _ _ _ ");
    putstr(&"\n| | (_) | | | | ");
    putstr(&"\n| | ____ ___ ____ _____| |_____ ____ ____ _____| | ");
    putstr(&"\n| |/ ___) _ \\| _ \\ | _ _) ___ |/ ___) _ \\| ___ | | ");
    putstr(&"\n| | | | |_| | | | | | | \\ \\| ____| | | | | | ____| | ");
    putstr(&"\n|_|_| \\____/|_| |_| |_| \\_\\_____)_| |_| |_|_____)__)\n\n");
}