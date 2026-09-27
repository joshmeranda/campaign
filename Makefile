.venv:
	python3 -m venv .venv; \
	. .venv/bin/activate; \
	pip install bs4 requests

assets/spells.csv: .venv
	mkdir --parents assets
	. .venv/bin/activate; \
	.venv/bin/python3 hacking/collect-spells.py

assets: assets/spells.csv

.PHONY: clean
clean:
	rm --recursive assets .venv