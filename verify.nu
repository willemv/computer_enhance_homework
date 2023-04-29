#!/usr/bin/env nu

ls assignments\*.asm | each { |it|
   target\debug\computer_enhance.exe $it.name | save --force scratch\out.bin ; {name: $it.name exit: $env.LAST_EXIT_CODE}
}
ls assignments\listing_0055_challenge_rectangle.asm  | path basename -c [ name ] | each { |it| let bin = (['assignments' $it.name] | path join | str substring 0..-4); target\debug\computer_enhance.exe $bin | save --force out.bin ; {name: $bin, exit: $env.LAST_EXIT_CODE} }