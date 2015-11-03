PHONY: all

OCB = ocamlbuild -use-ocamlfind

all: edit.native

edit.native: edit.ml
	$(OCB) edit.native

aedit.native: aedit.ml
	$(OCB) aedit.native

go: edit.native
	./edit.native

inc: aedit.native
	./aedit.native

clean:
	ocamlbuild -clean