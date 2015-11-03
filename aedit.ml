(*

Modification of edit.ml to incrementalize with Adapton

TODO: deal with insert keypress semantics, currently doing two commands
TODO: reorganize into modules

*)
open Graphics
open Adapton

(* labeled edit point *)
type cursor = string

module ArtLib = Insts.Nominal
type name = Name.t

(* list with edit points *)
type 'a cdata =
| Cursor of cursor
| Data of 'a
module CData(D : Data.S) : Data.S =
struct
    type t = D.t cdata
    let hash seed =
    	function
    	| Cursor(c) -> Types.String.hash (Hashtbl.seeded_hash seed `Cursor) c
    	| Data(d) -> D.hash (Hashtbl.seeded_hash seed `Data) d
    let compare t t' = compare (hash 42 t) (hash 42 t')
    let equal x x' =
    	x == x' ||
    	match x, x' with
        | Cursor(c), Cursor(c') -> Types.String.equal c c'
        | Data(d), Data(d') -> D.equal d d'
        | _ -> false
    let sanitize x =
    	match x with
      	| Cursor(c) -> Cursor(Types.String.sanitize c)
      	| Data(d) -> Data(D.sanitize d)
    let show x =
    	match x with
      	| Cursor(c) -> "Cursor("^Types.String.show c^")"
      	| Data(d) -> "Data("^(D.show d)^")"
    let pp fmt s = Format.fprintf fmt "%s" (show s)
end
module CharCList = SpreadTree.MakeSeq(Insts.Nominal)(Name)(CData(Types.Char))

(* cursor zipper *)
type 'a zip = 'a cdata list * cursor * 'a cdata list
module CharCZip = Types.Tuple3(CharCList)(Types.String)(CharCList)

(* directions for cursor manipulation *)
type dir = L | R
module Dir : Data.S =
struct
    type t = dir
    let hash = Hashtbl.seeded_hash
    let compare = compare
    let equal = (==)
    let show d =
    	match d with
    	| L -> "Left"
    	| R -> "Right"
    let pp fmt s = Format.fprintf fmt "%s" (show s)
    let sanitize x = x
end


(* commands for cursors *)
type 'a command =
	(* edits *)
| Replace of 'a * Dir.t
| Insert of 'a * Dir.t
| Remove of Dir.t
	(* cursor manipulation *)
| Move of Dir.t
| JumpTo of cursor
| SwitchTo of cursor
| Make of cursor

module Command(Elm : Data.S) : Data.S =
struct
    type t = Elm.t command
    let hash seed =
    	function
    	| Replace(e,d) -> Dir.hash (Elm.hash (Hashtbl.seeded_hash seed `Replace) e) d
    	| Insert(e,d) -> Dir.hash (Elm.hash (Hashtbl.seeded_hash seed `Insert) e) d
    	| Remove(d) -> Dir.hash (Hashtbl.seeded_hash seed `Remove) d
    	| Move(d) -> Dir.hash (Hashtbl.seeded_hash seed `Move) d
    	| JumpTo(c) -> Types.String.hash (Hashtbl.seeded_hash seed `JumpTo) c
    	| SwitchTo(c) -> Types.String.hash (Hashtbl.seeded_hash seed `SwitchTo) c
    	| Make(c) -> Types.String.hash (Hashtbl.seeded_hash seed `Make) c
    let compare t t' = compare (hash 42 t) (hash 42 t')
    let equal x x' =
    	x == x' ||
    	match x, x' with
        | Replace(e,d), Replace(e',d') -> Elm.equal e e' && Dir.equal d d'
        | Insert(e,d), Insert(e',d') -> Elm.equal e e' && Dir.equal d d'
        | Remove(d), Remove(d') -> Dir.equal d d'
        | Move(d), Move(d') -> Dir.equal d d'
        | JumpTo(c), JumpTo(c') -> Types.String.equal c c'
        | SwitchTo(c), SwitchTo(c') -> Types.String.equal c c'
        | Make(c), Make(c') -> Types.String.equal c c'
        | _ -> false
    let sanitize x =
    	match x with
    	| Replace(e,d) -> Replace(Elm.sanitize e, Dir.sanitize d)
    	| Insert(e,d) -> Insert(Elm.sanitize e, Dir.sanitize d)
    	| Remove(d) -> Remove(Dir.sanitize d)
    	| Move(d) -> Move(Dir.sanitize d)
    	| JumpTo(c) -> JumpTo(Types.String.sanitize c)
    	| SwitchTo(c) -> SwitchTo(Types.String.sanitize c)
    	| Make(c) -> Make(Types.String.sanitize c)
    let show x =
    	match x with
    	| Replace(e,d) -> "Replace("^Elm.show e^", "^Dir.show d^")"
    	| Insert(e,d) -> "Insert("^Elm.show e^", "^Dir.show d^")"
    	| Remove(d) -> "Remove("^Dir.show d^")"
    	| Move(d) -> "Move("^Dir.show d^")"
    	| JumpTo(c) -> "JumpTo("^Types.String.show c^")"
    	| SwitchTo(c) -> "SwitchTo("^Types.String.show c^")"
    	| Make(c) -> "Make("^Types.String.show c^")"
    let pp fmt s = Format.fprintf fmt "%s" (show s)
end

module ComCList = SpreadTree.MakeSeq(Insts.Nominal)(Name)(CData(Command(Types.Char)))
module HZip = Tuple3(ComCList)(Types.String)(ComCList)
(* actions for editor *)
type 'a action = 
| Command of 'a command
| Undo
| Redo

(* we store commands, convert to content later *)
(* 
type history = hzip
type content = czip
 *)

let print_clist cl =
	CharCList.simple_full_string cl

let other_dir dir =
	match dir with
	| L -> R
	| R -> L

let rev_clist clist =
	CharCList.list_reverse_balanced clist `Nil

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
	let r = CharCList.list_reverse_balanced l r in
	('Nil, c, r)

(* incrementalize *)
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
	(
		match d, l, r with
		| L, [], _ | R, _, [] -> content (* failwith "no value to replace" *)
		| L, Cursor(cur)::rest, _ -> do_command c (rest, user_c, Cursor(cur)::r)
		| R, _, Cursor(cur)::rest -> do_command c (Cursor(cur)::l, user_c, rest)
		| L, Data(_)::rest, _ -> (Data(new_item)::rest, user_c, r)
		| R, _, Data(_)::rest -> (l, user_c, Data(new_item)::rest)
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

(* incrementalize, commands is rope *)
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

let break_into_lines clist =
	let rec loop inp current outp =
		match inp with
		| [] -> current::outp
		| Cursor(c)::rest -> loop rest ("(" ^ c ^ ")" ^ current) outp
		| Data('\n')::rest | Data('\r')::rest -> loop rest "" (current::outp)
		| Data(d)::rest -> loop rest ((String.make 1 d) ^ current) outp
	in
	loop clist "" []

let draw_cursor () =
	rmoveto 2 (-2);
	rlineto 0 17;
	rmoveto 2 (-15)

let draw_string_list l =
	let rec draw count l =
		if count > 20 then () else
		match l with
		| [] -> ()
		| h::[] -> draw_string h
		| h::t ->
		draw_string h;
		moveto 2 ((current_y()) - 15);
		draw (count + 1) t
	in
	draw 0 l

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
	let r = rev_clist r in
	let r_lines = break_into_lines r in
	let l_lines = break_into_lines l in
	(l_lines, r_lines)
	
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
	(* act on keypress *)
	match int_of_char(read_key()) with
	|(* esc *) 27 -> () 
	|(* ctrl-z *) 26 -> history |> do_action Undo |> loop
	|(* ctrl-y *) 25 -> history |> do_action Redo |> loop
	|(* ctrl-x switch *) 24 -> 
		let c = read_key() in
		history |> do_action (Command(SwitchTo(String.make 1 c))) |> loop
	|(* ctrl-c jump *) 3 ->
		let c = read_key() in
		history |> do_action (Command(JumpTo(String.make 1 c))) |> loop
	|(* ctrl-v create *) 22 -> 
		let c = read_key() in
		history |> do_action (Command(Make(String.make 1 c))) |> loop
	|(* delete *) 8 -> history |> do_action (Command(Remove(other_dir dir))) |> loop
	|(* TODO: Move Up *) 9 -> history |> loop
	|(* Move Left *) 10 -> history |> do_action (Command(Move(L))) |> loop
	|(* TODO: Move Down *) 11 -> history |> loop
	|(* Move Right *) 12 -> history |> do_action (Command(Move(R))) |> loop
	|(* ctrl-o overwrite mode *) 15 -> history |> repl dir (not mode)
	|(* ctrl-u direction mode *) 21 -> history |> repl (other_dir dir) mode
	| ascii -> 
	let k = char_of_int ascii in
	match mode with
	| true -> 
		history
		|> do_action (Command(Replace(k, dir)))
		|> do_action (Command(Move(dir)))
	    |> loop
	| false -> 
		history 
		|> do_action (Command(Insert(k,dir)))
		|> do_action (Command(Move(dir)))
		|> loop

let user_repl () =
	open_graph ":0.0 800x600+0-0";
	repl R false (blank_editor None)

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