PHONY: all

OCB = ocamlbuild -use-ocamlfind

all: edit.native

edit.native: edit.ml
	$(OCB) edit.native

go: edit.native
	./edit.native

clean:
	ocamlbuild -clean