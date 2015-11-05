(*
TODO: deal with insert keypress semantics, currently doing two commands
TODO: reorganize into modules

*)
open Graphics

(*
(* This should work if adapton is installed correctly *)
open Adapton
module Seq = SpreadTree.MakeSeq(Insts.Nominal)(Name)(Types.Int)
*)

(* labeled edit point *)
type cursor = string

(* list with edit points *)
type 'a cdata =
| Cursor of cursor
| Data of 'a

(* zipper for clists *)
type 'a zip = 'a cdata list * cursor * 'a cdata list

(* directions for cursor manipulation *)
type dir = L | R

(* commands for cursors *)
type 'a command =
	(* edits *)
| Replace of 'a * dir
| Insert of 'a * dir
| Remove of dir
	(* cursor manipulation *)
| Move of dir
| JumpTo of cursor
| SwitchTo of cursor
| Make of cursor

(* actions for editor *)
type 'a action = 
| Command of 'a command
| Undo
| Redo

(* we store commands, convert to content later *)
type 'a history = 'a command zip
type 'a content = 'a zip

let print_clist cl =
	let rec p cl =
		match cl with
		| [] -> print_string "]"
		| Cursor(c)::r -> 
			print_string ("Cursor(" ^ c ^ "); ");
			p r
		| Data(d)::r ->
			print_string ("Data; ");
			p r
	in
	print_string "[";
	p cl;
	print_string "\n"

let other_dir dir =
	match dir with
	| L -> R
	| R -> L

let rev_clist clist =
	let rec rev h t =
		match h with
		| [] -> t
		| Cursor(c)::rest -> rev rest (Cursor(c)::t)
		| Data(d)::rest -> rev rest (Data(d)::t)
	in
	rev clist []

let blank_editor cursor = 
	match cursor with
	| None -> ([], "undo_location", [])
	| Some(cursor) -> 
	(* read bottom-up *)
	let gen_cur =
		[Data(SwitchTo(cursor))
		;Data(Make(cursor))
		]
	in
	(gen_cur, "undo_location", [])

let zip_to_left (l,c,r) =
	let rec flip_left l r =
		match l with
		| [] -> ([], r)
		| Cursor(c)::rest -> flip_left rest (Cursor(c)::r)
		| Data(d)::rest -> flip_left rest (Data(d)::r)
	in
	let (l', r') = flip_left l r in
	(l', c, r')


let zip_to_cursor_no_save (l,c,r) target =
	(* print_clist l; *)
	(* print_clist r; *)
	let rec flip_to_cursor in_l out_l target =
		match in_l with
		| [] -> None
		| Cursor(cur)::rest -> 
			(* print_string ("(cur: "^cur^" tar: "^target^")"); *)
			if cur = target then Some(rest, out_l) else
			flip_to_cursor rest (Cursor(cur)::out_l) target
		| Data(d)::rest -> flip_to_cursor rest (Data(d)::out_l) target
	in
	if target = c then (l,c,r) else
	(* search left *)
	match flip_to_cursor l r target with
	| Some(l', r') -> (l', target, r')
	| None ->
	(* search right *)
	match flip_to_cursor r l target with
	| Some(r', l') -> (l', target, r')
	| None -> failwith ("cursor not found: " ^ target)

let zip_to_cursor (l,c,r) target = 
	if target = c then (l,c,r) else
	zip_to_cursor_no_save (l,c,(Cursor(c)::r)) target


let rec do_command c content =
	let (l, user_c, r) = content in
	match c with
	| Replace(new_item, d) ->
	(* modified to be Overwrite *)
	(
		match d, l, r with
		(* do an insert at the end of the text *)
		| L, [], _ -> (l, user_c, Data(new_item)::r)
		| R, _, [] -> (Data(new_item)::l, user_c, r)

		| L, Cursor(cur)::rest, _ -> do_command c (rest, user_c, Cursor(cur)::r)
		| R, _, Cursor(cur)::rest -> do_command c (Cursor(cur)::l, user_c, rest)
		| L, Data(_)::rest, _ -> (rest, user_c, Data(new_item)::r)
		| R, _, Data(_)::rest -> (Data(new_item)::l, user_c, rest)
	) 
	| Insert(new_item, d) ->
	(
		match d, l, r with
		| L, Cursor(cur)::rest, _ -> do_command c (rest, user_c, Cursor(cur)::r)
		| R, _, Cursor(cur)::rest -> do_command c (Cursor(cur)::l, user_c, rest)
		| L, _, _ -> (Data(new_item)::l, user_c, r)
		| R, _, _ -> (l, user_c, Data(new_item)::r)
	)
	| Remove(d) ->
	(
		match d, l, r with
		| L, [], _ | R, _, [] -> content (* failwith "no value to remove" *)
		| L, Cursor(cur)::rest, _ -> do_command c (rest, user_c, Cursor(cur)::r)
		| R, _, Cursor(cur)::rest -> do_command c (Cursor(cur)::l, user_c, rest)
		| L, Data(_)::rest, _ -> (rest, user_c, r)
		| R, _, Data(_)::rest -> (l, user_c, rest)
	)
	(* TODO: try to collapse the Move command into one case *)
	| Move(L) ->
	(
		match l with
		| [] -> (l,user_c,r)
		| Cursor(cur)::rest -> do_command c (rest, user_c, Cursor(cur)::r)
		| Data(d)::rest -> (rest, user_c, Data(d)::r)
	)
	| Move(R) -> 
	(
		match r with
		| [] -> (l,user_c,r)
		| Cursor(cur)::rest -> do_command c (Cursor(cur)::l, user_c, rest)
		| Data(d)::rest -> (Data(d)::l, user_c, rest)
		
	)
	| JumpTo(cur) ->
 		(* Printf.printf "Cursor: %s, JumpTo: %s\n" user_c cur; *)
		if user_c = cur then content else
		let (l',c',r') = zip_to_cursor_no_save content cur in
		(l', user_c, Cursor(c')::r')
	| SwitchTo(cur) -> 
 		(* Printf.printf "Cursor: %s, SwitchTo: %s\n" user_c cur; *)
		zip_to_cursor content cur
	| Make(cur) ->
 		(* Printf.printf "Cursor: %s, Make: %s\n" user_c cur; *)
		(l,user_c,Cursor(cur)::r)

let do_action a history =
	match a with
	| Command(c) ->
		let (old_l,user_c,new_l) = history in
		do_command (Insert(c,L)) (old_l,user_c,[])
	| Undo -> do_command (Move(L)) history
	| Redo -> do_command (Move(R)) history

let build_content history =
	let rec build_content content commands = 
		match commands with
		| [] -> content
		| Cursor(_)::rest ->
			build_content content rest
		| Data(command)::rest ->
			build_content (do_command command content) rest
	in
	let (commands, _, _) = history in
	build_content ([], "0", []) (rev_clist commands)

let print_content content =
	let (_,_,content) = zip_to_left content in
	let rec print_content content =
		match content with
		| [] -> print_string "\n"
		| Cursor(c)::rest -> print_content rest
		| Data(d)::rest -> 
			(print_string d; print_content rest)
	in
	print_content content

let do_input history input =
	let rec do_input history input =
		match input with
		| [] -> history
		| a::rest -> do_input (do_action a history) rest
	in
	do_input history input

let break_into_lines clist =
	let rec loop count inp current outp =
		if count = 20 then outp else
		match inp with
		| [] -> current::outp
		| Cursor(c)::rest -> loop count rest ("(" ^ c ^ ")" ^ current) outp
		| Data('\n')::rest | Data('\r')::rest -> loop (count + 1) rest "" (current::outp)
		| Data(d)::rest -> loop count rest ((String.make 1 d) ^ current) outp
	in
	loop 0 clist "" []

let break_into_lines_r clist =
	let rec loop count inp current outp =
		if count = 21 then outp else
		match inp with
		| [] -> current::outp
		| Cursor(c)::rest -> loop count rest (current ^ "(" ^ c ^ ")") outp
		| Data('\n')::rest | Data('\r')::rest -> loop (count + 1) rest "" (current::outp)
		| Data(d)::rest -> loop count rest (current ^ (String.make 1 d)) outp
	in
	loop 0 clist "" []

let draw_cursor () =
	rmoveto 2 (-2);
	rlineto 0 17;
	rmoveto 2 (-15)

let rec draw_string_list l =
	match l with
	| [] -> ()
	| h::[] -> draw_string h
	| h::t ->
	draw_string h;
	moveto 2 ((current_y()) - 15);
	draw_string_list t

let print_string_list l =
	let rec print count l =
		match l with
		| [] -> ()
		| h::[] -> print_string h
		| h::t ->
		print_string h;
		print_string "\n";
		print (count + 1) t
	in
	print 0 l

let lines_of_content content =
	let (l,_,r) = content in
	let l_lines = break_into_lines l in
	let r_lines = break_into_lines_r r in
	(l_lines, List.rev r_lines)
	
let saved_actions = ref []
let save act_string = 
	saved_actions := act_string::!saved_actions

let draw_saved_actions() =
	let rec draw_act c l =
		if c > 40 then () else
		match l with
		| [] -> ()
		| a::t -> 
			draw_string a;
			moveto 600 ((current_y()) + 15);
			draw_act (c+1) t
	in
	draw_act 0 !saved_actions

let rec repl dir mode history =
	let loop = repl dir mode in 
	(* draw history to screen *)
	let (l_lines,r_lines) = 
		history
		|> build_content
		|> lines_of_content
	in
	clear_graph();moveto 2 585;
	draw_string_list l_lines;
	draw_cursor();
	draw_string_list r_lines;
	moveto 600 2;
	draw_saved_actions();
	(* act on keypress *)
	match int_of_char(read_key()) with
	|(* esc *) 27 -> () 
	|(* ctrl-z *) 26 -> save "Undo"; history |> do_action Undo |> loop
	|(* ctrl-y *) 25 -> save "Redo"; history |> do_action Redo |> loop
	|(* ctrl-x switch *) 24 -> 
		let c = read_key() in
		let c = String.make 1 c in
		save ("Switch to ("^c^")");
		history |> do_action (Command(SwitchTo(c))) |> loop
	|(* ctrl-c jump *) 3 ->
		let c = read_key() in
		let c = String.make 1 c in
		save ("Jump to ("^c^")");
		history |> do_action (Command(JumpTo(c))) |> loop
	|(* ctrl-v create *) 22 -> 
		let c = read_key() in
		let c = String.make 1 c in
		save ("Create ("^c^")");
		history |> do_action (Command(Make(c))) |> loop
	|(* delete *) 8 -> save "Backspace"; history |> do_action (Command(Remove(other_dir dir))) |> loop
	|(* TODO: Move Up *) 9 -> save "Cursor up (unimplemented)"; history |> loop
	|(* Move Left *) 10 -> save "Cursor left"; history |> do_action (Command(Move(L))) |> loop
	|(* TODO: Move Down *) 11 -> save "Cursor down (unimplemented)"; history |> loop
	|(* Move Right *) 12 -> save "Cursor right"; history |> do_action (Command(Move(R))) |> loop
	|(* ctrl-o overwrite mode *) 15 -> save "Switch overwrite mode"; history |> repl dir (not mode)
	|(* ctrl-u direction mode *) 21 -> save "Switch direction mode"; history |> repl (other_dir dir) mode
	| ascii ->
	let k = char_of_int ascii in
	match mode with
	| true ->
		if ascii = 13 then save "Replace with Return" else save ("Replace with "^(String.make 1 k));
		history
		|> do_action (Command(Replace(k, dir)))
	    |> loop
	| false ->
		if ascii = 13 then save "Insert Return" else save ("Insert "^(String.make 1 k));
		history 
		|> do_action (Command(Insert(k,(other_dir dir))))
		|> loop

let time thunk =
	let start = Unix.gettimeofday () in
	let res = thunk() in
    let stop = Unix.gettimeofday () in
    let t = (stop -. start) in
    (t,res)

let random_commands num =
	let add_rnd_act cursor_count history =
		(* print_int cursor_count; *)
		let rnd_cursor() =
			string_of_int (Random.int cursor_count)	
		in
		let rnd_char() =
			char_of_int (
				match Random.int 20 with
				| (* space *) n when n < 5 -> 32
				| (* numbers *) n when n < 7 -> (Random.int 10) + 48
				| (* lower case *) n when n < 17 -> (Random.int 26) + 97
				| (* upper case *) n when n < 19 -> (Random.int 26) + 65
				| (* return *) _ -> 13
			)
		in
		let rnd_dir() =
			if Random.bool() then R else L
		in
		let (c, act) =
			match Random.int 100 with
			| n when n < 18 -> (cursor_count, Command(Replace(rnd_char(),rnd_dir())))
			| n when n < 63 -> (cursor_count, Command(Insert(rnd_char(),rnd_dir())))
			| n when n < 81 -> (cursor_count, Command(Remove(rnd_dir())))
			| n when n < 99 -> (cursor_count, Command(Move(R)))
			| _ ->
			match Random.int 3 with
			| 0 -> (cursor_count + 1, Command(Make(string_of_int cursor_count)))
			| 1 -> (cursor_count, Command(SwitchTo(rnd_cursor())))
			| 2 -> (cursor_count, Command(JumpTo(rnd_cursor())))
			| _ -> (cursor_count, Undo)
		in
		(c, do_action act history)
	in
	let rec loop n (c,h) =
		if n = 0 then (c,h) else
		add_rnd_act c h |> loop (n - 1)
	in
	let (_,ret) = loop num (1, blank_editor None) in
	ret

let user_repl () =
	open_graph ":0.0 800x600+0-0";
	repl R false (random_commands 100000)(* (blank_editor None) *)

let _ = user_repl()

(* 
let default_filename = "result_data.csv"
let print_generated_text = false
let base_number_of_generated_commands = 100000
let _ =
	let out_file = open_out_gen [Open_creat;Open_append] 0o666 default_filename in
	Printf.fprintf out_file "commands, generate, build_content, print_prep, print\n%!";
	for n = 1 to 10 do
		Random.self_init();
		let (t0,rnd) = time (fun () -> random_commands (n * base_number_of_generated_commands)) in
		let (t1,content) = time (fun () -> build_content rnd) in
		let (t2,(lls,rls)) = time (fun () -> lines_of_content content) in
		let (t3,_) = 
			if not print_generated_text then (0.,()) else
			time (fun () ->
				let (_,cur,_) = content in 
				print_string "\n============================================\n";
				print_string_list lls;
				print_string ("(c:"^cur^")");
				print_string_list rls;
				print_string "\n============================================\n";
				print_newline();
			)
		in
		Printf.fprintf out_file "%d, %1.4f, %1.4f, %1.4f, %1.4f\n%!"
			(n * base_number_of_generated_commands)
			t0 t1 t2 t3
	done

 *)