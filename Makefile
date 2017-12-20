.PHONY: rebuild-tocs

# note: with this target make compains about circular deps but hey, it drops them too that it's exactly what I wanted
%/README.md: %/*.md
	./gen-section-toc.sh $(@D) > $@

README.md: $(wildcard */README.md) README.prefix.md
	./gen-toc.sh > $@

