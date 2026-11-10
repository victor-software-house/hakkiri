// Boundary F1 of prose's sentence segmenter, as Vale 3.26.0 builds it (segment.New),
// on fixtures in the splitter format and on CoNLL-U files.
//
//	go run . fixture NAME PATH   record = "# source" line, sentences one per line, blank line
//	go run . conllu  NAME PATH   record = consecutive sentences of one # newdoc / # newpar block
//
// Text of a record is its sentences joined by one space. Gold boundaries are the end
// offsets of every sentence but the last; the end of the text is a boundary by definition.
package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"

	"github.com/jdkato/prose/v3/segment"
)

type record []string

func readFixture(path string) ([]record, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	var recs []record
	var cur record
	sc := bufio.NewScanner(f)
	sc.Buffer(make([]byte, 1<<20), 1<<20)
	flush := func() {
		if len(cur) > 0 {
			recs = append(recs, cur)
			cur = nil
		}
	}
	for sc.Scan() {
		line := sc.Text()
		switch {
		case line == "":
			flush()
		case strings.HasPrefix(line, "# "):
			flush()
		default:
			cur = append(cur, line)
		}
	}
	flush()
	return recs, sc.Err()
}

func readConllu(path string) ([]record, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	var recs []record
	var cur record
	sc := bufio.NewScanner(f)
	sc.Buffer(make([]byte, 1<<20), 1<<20)
	flush := func() {
		if len(cur) > 0 {
			recs = append(recs, cur)
			cur = nil
		}
	}
	for sc.Scan() {
		line := sc.Text()
		switch {
		case strings.HasPrefix(line, "# newdoc"), strings.HasPrefix(line, "# newpar"):
			flush()
		case strings.HasPrefix(line, "# text = "):
			cur = append(cur, strings.TrimPrefix(line, "# text = "))
		}
	}
	flush()
	return recs, sc.Err()
}

func main() {
	if len(os.Args) != 4 {
		fmt.Fprintln(os.Stderr, "usage: segf1 fixture|conllu NAME PATH")
		os.Exit(2)
	}
	kind, name, path := os.Args[1], os.Args[2], os.Args[3]
	read := map[string]func(string) ([]record, error){"fixture": readFixture, "conllu": readConllu}[kind]
	if read == nil {
		fmt.Fprintln(os.Stderr, "unknown kind", kind)
		os.Exit(2)
	}
	recs, err := read(path)
	if err != nil {
		panic(err)
	}
	seg, err := segment.New()
	if err != nil {
		panic(err)
	}
	var sents, gold, pred, tp int
	for _, r := range recs {
		text := strings.Join(r, " ")
		g := map[int]bool{}
		off := 0
		for i, s := range r {
			off += len(s)
			if i < len(r)-1 {
				g[off] = true
			}
			off++
		}
		p := map[int]bool{}
		ss := seg.Segment(text)
		for i, s := range ss {
			if i < len(ss)-1 {
				p[s.End()] = true
			}
		}
		sents += len(r)
		gold += len(g)
		pred += len(p)
		for k := range p {
			if g[k] {
				tp++
			}
		}
	}
	prec, rec, f1 := 0.0, 0.0, 0.0
	if pred > 0 {
		prec = float64(tp) / float64(pred)
	}
	if gold > 0 {
		rec = float64(tp) / float64(gold)
	}
	if prec+rec > 0 {
		f1 = 2 * prec * rec / (prec + rec)
	}
	fmt.Printf("%s\t%d\t%d\t%d\t%d\t%d\t%.4f\t%.4f\t%.4f\n", name, len(recs), sents, gold, pred, tp, prec, rec, f1)
}
