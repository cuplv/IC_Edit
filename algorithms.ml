
(* Adapton Primitives; not "user code" *)
module APrimitives : sig
    type name
    type 'a art
    val fork : name -> name * name
    val pair : name -> name -> name
    val thunk : name -> (unit -> 'a) -> 'a art
    val cell  : name -> 'a -> 'a art
    val force : 'a art -> 'a
  end
  =
  struct
    type name =
      | Root
      | ForkL of name
      | ForkR of name
      | Pair  of name * name

    let fork : name -> name * name = fun n -> (ForkL n, ForkR n)
    let pair : name -> name -> name = fun n1 n2 -> Pair(n1, n2)
                                                       
    (* 'a art =def= "thunk returning 'a" UNION "cell holding 'a" *)
    type 'a art =
      | Art of 'a  

    let thunk : name -> (unit -> 'a) -> 'a art = fun _ -> failwith "TODO"
    let cell  : name ->          'a  -> 'a art = fun _ -> failwith "TODO"
    let force : 'a art -> 'a                   = fun _ -> failwith "TODO"
  end

open APrimitives

module ICEditAlgo = struct
    type lev = int (* A "level", used in Pugh's hash-tree representation *)
    type dir = L | R

    type 'a tree =
      | Nil
      | Leaf of 'a
      | Bin  of lev * name option * 'a tree * 'a tree
      | Art  of ('a tree) art

    type 'a tlist =
      | Nil
      | Cons of 'a * 'a tlist
      | Name of name * 'a tlist
      | Art  of ('a tlist) art
      | Tree of dir * ('a tree) (* Can we drop this 'dir' part of the Tree constructor? *)

    type 'a raz = ('a tlist * 'a tlist)
    type cur = string
    type dat = string
    type sym = Cur of cur | Dat of dat

    type info = { curs : cur list ; size : int }                                     
    let rec tree_info : sym tree -> info =
      fun _ -> failwith "TODO-Much-Later"
                        
    let rec tree_of_list : sym tlist -> sym tree =
      fun _ -> failwith "TODO-Later" (* Question: Do we need a trampoline thing here? *)
                        
    let rec focus : sym tree -> cur -> (sym raz) option =
      fun _ -> failwith "TODO-Now"

    let rec expose : 'a tlist -> 'a tlist =
      fun _ -> failwith "TODO-Now"

    let rec unfocus : sym raz -> sym tree=
      fun _ -> failwith "TODO-Now"
                        
    let rec insert : sym -> sym raz -> dir -> sym raz =                                                    
      fun _ -> failwith "TODO-Later"

    let rec remove : sym -> sym raz -> dir -> sym raz =
      fun _ -> failwith "TODO-Later"

    let rec tree_append : name option -> 'a tree -> 'a tree -> 'a tree =
      fun no t1 t2 -> let t_out = match (t1, t2) with
      | Nil, _ -> t2 | _, Art(a) -> tree_append None t2 (force a)
      | _, Nil -> t1 | Art(a), _ -> tree_append None (force a) t2
      | Leaf(_), Leaf(_) -> Bin(0, None, t1, t2)
      | Leaf(_), Bin(lv,n,l,r) -> Bin(lv, n, tree_append n t1 l, r)
      | Bin(lv,n,l,r), Leaf(_) -> Bin(lv, n, l, tree_append n r t2)
      | Bin(lv1,n1,t1l,t1r), Bin(lv2,n2,t2l,t2r) -> if lv1 > lv2
        then Bin(lv1, n1, t1l, tree_append n1 t1r t2)
        else Bin(lv2, n2, tree_append n2 t1 t2l, t2r)
      in match no with None -> t_out | Some(n) -> Art(cell n t_out)

  end
       
module New = struct (* For discussion on Monday, March 7 2016 *)

    type 'a alist =
      | Nil
      | Cons of 'a * 'a alist
      | Name of name * 'a alist
      | Art  of ('a alist) art

    type lev = int (* A "level", used in Pugh's hash-tree representation *)
    type 'a atree =
      | Nil
      | Leaf of 'a
      | Bin  of lev * 'a atree * 'a atree
      | Name of lev * name * 'a atree * 'a atree
      | Art  of ('a atree) art

    let tree_fold_up : 'a atree -> ('a option -> 'b) -> ('b -> 'b -> 'b) -> 'b =
      fun t base_case binary_case ->
      let rec recur t =
        match t with
        | Nil        -> base_case None
        | Leaf(x)    -> base_case (Some x)
        | Bin(_,l,r) -> binary_case (recur l) (recur r)
        | Name(_,n,l,r) ->
           let nl, nr = fork n in
           let resl   = force (thunk nl (fun () -> recur l)) in
           let resr   = force (thunk nr (fun () -> recur r)) in
           binary_case resl resr
        | Art(a) -> recur (force a)
      in recur t

    let tree_fold_lr : 'a atree -> 'b -> ('b -> name option -> 'a -> 'b) -> ('b * name option) =
      fun t start leaf_case ->
      let rec recur t (accum:'b) (nameopt:name option) : ('b * name option) =
        match t with
        | Nil        -> (accum, nameopt)
        | Leaf(leaf) -> (leaf_case accum nameopt leaf, None)
        | Bin(_,l,r) -> let accum, nameopt = recur l accum nameopt in
                        let accum, nameopt = recur r accum nameopt in
                        (accum, nameopt)
        | Name(_,n,l,r) ->
           let nl, nr   = fork n in
           let accum, nameopt = force (thunk nl (fun () -> recur l accum (Some n))) in
           let accum, nameopt = force (thunk nr (fun () -> recur l accum nameopt )) in
           (accum, nameopt)
        | Art(a) -> recur (force a) accum nameopt
      in
      recur t start None
  end




module Old = struct
(* - - - - - - - - - - - - - - - - - - - - - - - - -  *)
(* Collection library; consists only of "user code"   *)
      
type 'a alist =
  | Nil
  | Cons of 'a * 'a alist
  | Name of name * 'a alist
  | Art  of ('a alist) art

type lev = int (* A "level", used in Pugh's hash-tree representation *)
type 'a atree =
  | Nil
  | Leaf of 'a
  | Bin  of lev * 'a atree * 'a atree
  | Name of lev * name * 'a atree * 'a atree
  | Art  of ('a atree) art
 
(* This version does not use fork; it consumes and produces the same names as the input list *)
let rec list_map :  ('a -> 'b) -> 'a alist -> 'b alist =
  fun mapf list ->
  match list with
  | Nil -> Nil
  | Cons(x,list') -> Cons(mapf x, list_map mapf list')

  | Art a -> list_map mapf (force a)
  | Name(n,list') -> Name(n, Art(thunk n (fun _ -> list_map mapf list')))

(* This version uses fork to consume and produce distinct names from the input list *)
let rec list_map' :  ('a -> 'b) -> 'a alist -> 'b alist =
  fun mapf list ->
  match list with
  | Nil -> Nil
  | Cons(x,list') -> Cons(mapf x, list_map mapf list')

  | Art a -> list_map mapf (force a)                                               
  | Name(n,list') -> let (n1,n2) = fork n in
                     Name(n1, Art(thunk n2 (fun _ -> list_map mapf list')))
                      
let rec list_of_tree : 'a atree -> 'a alist -> 'a alist =
  fun tree rest ->
  match tree with
  | Nil -> Nil
  | Leaf x -> Cons(x, list_of_tree Nil rest)
  | Bin(_,t1,t2) -> list_of_tree t1 (list_of_tree t2 rest)
  | Name(_,n,t1,t2) ->
     Name(n, Art(thunk n (fun _ -> list_of_tree t1 (Art (thunk n (fun _ -> list_of_tree t2 rest))))))
  | Art a -> list_of_tree (force a) rest
                          
end
