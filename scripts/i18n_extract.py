#!/usr/bin/env python3
"""Sammelt alle deutschen Texte, die übersetzt werden: tr("…")/trf("…") der Oberfläche,
feste Meldungen des Servers (AppError::bad/forbidden/Conflict, Err("…")) und die Felder der
KI. Schreibt locales/strings.json (sortiert) und meldet je Sprache, was fehlt oder veraltet ist.

    scripts/i18n_extract.py           Liste neu schreiben, Stand je Sprache zeigen
    scripts/i18n_extract.py --check   nur prüfen (Exit 1, wenn eine Sprache unvollständig ist)
"""
import glob
import json
import os
import re
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
LIT = r'"((?:[^"\\]|\\.)*)"'


def unescape(s):
    return json.loads('"' + s + '"')


def ui_strings():
    out = set()
    for path in glob.glob(os.path.join(ROOT, "ui/src/**/*.rs"), recursive=True):
        src = open(path, encoding="utf-8").read()
        for m in re.finditer(r"\btrf?\(\s*" + LIT, src):
            out.add(unescape(m.group(1)))
        # Tabellen, die erst beim Anzeigen übersetzt werden
        for m in re.finditer(r'\("[^"]*",\s*' + LIT + r"\)", src):
            if re.search(r"(OPERATORS|PROPERTY_TYPES|WEBHOOK_EVENTS|PERMISSIONS|ITEMS)", src[: m.start()][-3000:]):
                out.add(unescape(m.group(1)))
        for m in re.finditer(r'(?:label|group):\s*' + LIT, src):
            out.add(unescape(m.group(1)))
    # match-Arme in Beschriftungs-Funktionen (tr(match … { "x" => "Text" }))
    models = open(os.path.join(ROOT, "ui/src/models.rs"), encoding="utf-8").read()
    for block in re.findall(r"crate::i18n::tr\(match[^}]*\}", models, re.S):
        for m in re.finditer(r"=>\s*" + LIT, block):
            out.add(unescape(m.group(1)))
    admin = open(os.path.join(ROOT, "ui/src/pages/settings/admin.rs"), encoding="utf-8").read()
    for m in re.finditer(r'=>\s*"(Lokal \(eigener Rechner\)|Eigene Adresse|Cloud)"', admin):
        out.add(m.group(1))
    return out


def server_strings():
    out = set()
    for path in glob.glob(os.path.join(ROOT, "src/**/*.rs"), recursive=True):
        src = open(path, encoding="utf-8").read()
        pats = [
            r"AppError::(?:bad|forbidden)\(\s*" + LIT,
            r"AppError::Conflict\(\s*" + LIT,
            r"AppError::unique\([^,]+,\s*" + LIT,
            r"Err\(\s*" + LIT + r"\s*\.into\(\)",
            r"Some\(\s*" + LIT + r"\s*\.into\(\)\)",
        ]
        if path.endswith("catchers.rs"):
            pats.append(r"\b(?:400|401|403|404|413|422|_) => " + LIT)
        for p in pats:
            for m in re.finditer(p, src):
                s = unescape(m.group(1))
                if "{" not in s and re.search(r"[A-Za-zÄÖÜäöü]", s):
                    out.add(s)
    ai = open(os.path.join(ROOT, "src/ai.rs"), encoding="utf-8").read()
    for m in re.finditer(r'\("\w+",\s*' + LIT + r"\)", ai[ai.index("META_FIELDS"):]):
        out.add(unescape(m.group(1)))
    for m in re.finditer(r'(?:name|"[a-z]+", ")([^"]+\(lokal\)|Eigene OpenAI-kompatible API)"', ai):
        out.add(m.group(1))
    for m in re.finditer(r'"([^"]+\(lokal\))"', ai):
        out.add(m.group(1))
    return out


def main():
    strings = sorted(ui_strings() | server_strings())
    path = os.path.join(ROOT, "locales/strings.json")
    if "--check" not in sys.argv:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as f:
            json.dump(strings, f, ensure_ascii=False, indent=0)
            f.write("\n")
    print(f"{len(strings)} Texte")
    missing_any = False
    for lp in sorted(glob.glob(os.path.join(ROOT, "locales/*.json"))):
        code = os.path.basename(lp)[:-5]
        if code == "strings":
            continue
        cat = json.load(open(lp, encoding="utf-8"))
        missing = [s for s in strings if not cat.get(s)]
        stale = [k for k in cat if k not in strings]
        bad = [k for k, v in cat.items() if v and sorted(re.findall(r"\{\d+\}", k)) != sorted(re.findall(r"\{\d+\}", v))]
        if missing or bad:
            missing_any = True
        print(f"  {code}: {len(strings) - len(missing)}/{len(strings)}"
              + (f", {len(stale)} veraltet" if stale else "")
              + (f", {len(bad)} mit falschen Platzhaltern: {bad[:3]}" if bad else ""))
    if "--check" in sys.argv and missing_any:
        sys.exit(1)


if __name__ == "__main__":
    main()
