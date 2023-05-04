#!/usr/bin/env nu

cargo build --release --bin printer;

let r = (
   ls assignments\*.asm | each { |it|
      rm --force scratch\out.asm ;
      rm --force scratch\out ;
      let binary = ($it.name | str substring ..-4) ;
      print -n . ;
      target\release\printer.exe $binary out> scratch\out.asm err> scratch\err.log;
      let decode = $env.LAST_EXIT_CODE;
      ^nasm scratch\out.asm ;
      let nasm = $env.LAST_EXIT_CODE;
      ^fc $binary scratch\out
      let fc = $env.LAST_EXIT_CODE ;
      {name: $it.name decode: $decode nasm: $nasm compare: $fc}
   }
)
print ""
$r | table