#!/usr/bin/python3
import requests
from bs4 import BeautifulSoup
import csv
import time
import logging

DND_5E_ROOT: str = "https://dnd5e.wikidot.com"
ALL_SPELLS_ENDPOINT: str = "/spells:all"

SPELL_CSV = "assets/spells.csv"

logger = logging.getLogger(__name__)

logging.basicConfig(format='%(asctime)s %(message)s', datefmt='%m/%d/%Y %I:%M:%S %p', level=logging.INFO)


def spells():
	r = requests.get(DND_5E_ROOT + ALL_SPELLS_ENDPOINT)

	soup = BeautifulSoup(r.content, "html.parser")
	main_content = soup.find("div", class_="main-content-wrap")

	if main_content is None:
		raise Exception("main_content cannot be None")

	for row in main_content.find_all("div", class_="row"):
		name = row.h1.span.string

		if name in ["Site Navigation", "Spells by Class"]:
			continue

		yield name, [a.get("href") for a in row.find_all("a")]


def get_spell(endpoint: str):
	r = requests.get(DND_5E_ROOT + endpoint)
	soup = BeautifulSoup(r.content, "html.parser")

	try:
		v = {
			# "endpoint": endpoint,
			"name": soup.find("div", class_="page-title").span.string,
		}

		if v["name"] == "The page does not (yet) exist.":
			return None

		def get_source(d, p):
			# if "Source:" in p.string:
			# 	d["source"] = p.string
			pass

		def get_level(d, p):
			value = "0"

			if p.string.endswith("cantrip"):
				value = "0"
			elif p.string.startswith("1st-level"):
				value = "1"
			elif p.string.startswith("2nd-level"):
				value = "2"
			elif p.string.startswith("3rd-level"):
				value = "3"
			elif p.string.startswith("4th-level"):
				value = "4"
			elif p.string.startswith("5th-level"):
				value = "5"
			elif p.string.startswith("6th-level"):
				value = "6"
			elif p.string.startswith("7th-level"):
				value = "7"
			elif p.string.startswith("8th-level"):
				value = "8"
			elif p.string.startswith("9th-level"):
				value = "9"

			d["level"] = value
		
		def get_description(d, p):
			d["description"] = p.string

		def get_at_higher_levels(d, p):
			at_higher_levels = p.find(string="At Higher Levels.")

			if at_higher_levels is not None:
				d["at_higher_levels"] = at_higher_levels.next_element
			else:
				d["at_higher_levels"] = ""

		def sub_fields(d, p):
			v["casting_time"] = p.find(string="Casting Time:").next_element
			v["range"] = p.find(string="Range:").next_element
			v["components"] = p.find(string="Components:").next_element
			v["duration"] = p.find(string="Duration:").next_element

		cbs = [
			get_source,
			get_level,
			sub_fields,
			get_description,
			get_at_higher_levels,
		]

		for (i, p) in enumerate(soup.find(class_="main-content").find_all("p")):
			try:
				cbs[i](v, p)
			except IndexError:
				break
	except Exception as err:
		# print(str(r.content))
		raise err

	return v


def get_already_searched():
	endpoints = []

	try:
		with open(SPELL_CSV, "r") as csvfile:
			reader = csv.reader(csvfile)

			for row in reader:
				endpoints.append(row[0])
	except IOError:
		pass

	return endpoints


def main():
	logger.info("checking foir previously found spells")
	searched = get_already_searched()

	with open(SPELL_CSV, "w+") as csvfile:	
		writer = csv.writer(csvfile)

		writer.writerow(["name", "level", "casting_time", "range", "components", "duration", "description", "at_higher_levels"])

		for category, endpoints in spells():
			logger.info(f"collecting '{category}' spells")

			for spell_endpoint in endpoints:
				if spell_endpoint in searched:
					logger.info(f"skipping {spell_endpoint}")
					continue
				else:
					logger.info(f"attemping {spell_endpoint}")

				retry = True
				while retry:
					try:
						spell = get_spell(spell_endpoint)
					except Exception as err:
						logger.error(f"Encountered error: {err}, waiting and retrying")
						time.sleep(5)
						continue

					retry = False

					if spell is None:
						logger.warning(f"spell not described {spell_endpoint}")
					else:
						writer.writerow(spell.values())


if __name__ == "__main__":
	main()
