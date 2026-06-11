---
title: Über NavigaTUM
description: NavigaTUM ist ein von Studierenden für Studierende entwickeltes Open-Source-Tool, um sich an der Technischen Universität München (TUM) zurechtzufinden.
---

# Über NavigaTUM

NavigaTUM hilft dir, Räume, Gebäude und Standorte der [TUM](https://tum.de) zu finden - vom Stammgelände über Garching und Weihenstephan bis Heilbronn und Straubing.
Das Projekt wird von Studierenden für Studierende entwickelt, ist [Open Source](https://github.com/TUM-Dev/NavigaTUM) und kommt ohne Login, ohne Werbung und ohne Tracking aus.
Hinter dem Projekt steht keine offizielle Stelle der TUM, sondern eine Gruppe Freiwilliger des [OpenSource @ TUM e.V.](https://tum.dev) - und vielleicht bald auch du.

## Woher die Daten kommen

Gute Navigation steht und fällt mit guten Daten, deswegen kombinieren wir mehrere Quellen.
Die Grundlage bilden die Gebäude, Räume, Organisationen und Kalendereinträge aus [TUMonline](https://campus.tum.de).
Wir übernehmen diese Daten regelmäßig automatisiert, strukturieren sie aber teilweise um (zum Beispiel bei Gebäudekomplexen) und korrigieren Fehler.
Den alten TUM Roomfinder haben wir abgelöst; seine Lagepläne und Raum-Metadaten leben bei uns als archivierter Datenbestand weiter.
Vieles pflegen wir außerdem von Hand nach, oft auf Basis von Hinweisen aus der Community: Koordinaten, Namen, das Suchranking, Öffnungszeiten mit Quellenangabe je Eintrag und einige selbst gezeichnete Lagepläne.

Das Rückgrat unserer Karte ist [OpenStreetMap](https://www.openstreetmap.org).
Gebäudeumrisse, Innenraumdaten, Points of Interest und viele Öffnungszeiten stammen aus OSM - und das ist keine Einbahnstraße: Was wir vor Ort verbessern, tragen wir in OSM ein, sodass auch alle anderen Karten davon profitieren.
Kartendaten © OpenStreetMap-Mitwirkende.

Einige Inhalte sind live.
Haltestellen in Campusnähe und die Abfahrten auf den Detailseiten kommen vom community-betriebenen ÖPNV-Routing-Projekt [Transitous](https://transitous.org), die aktuelle Verfügbarkeit von Lernräumen von [IRIS](https://iris.asta.tum.de), der Lernraum-Anzeige der Studentischen Vertretung.
Die Speisepläne und Öffnungszeiten der Mensen liefert die [eat-api](https://github.com/TUM-Dev/eat-api), aufbereitet aus den Daten des Studierendenwerks München Oberbayern; die Informationen zu den Teilbibliotheken stammen von der [TUM Universitätsbibliothek](https://www.ub.tum.de/teilbibliotheken).

## Bilder

Die Fotos von Gebäuden und Räumen stammen von Mitgliedern der Community.
Zu jedem Bild speichern wir, wer es aufgenommen hat und unter welcher Lizenz es steht (zum Beispiel CC0 oder CC BY) - die Angaben kannst du dir direkt am Bild anzeigen lassen.
Du warst mit der Kamera unterwegs? Über den Bearbeiten-Knopf auf jeder Detailseite kannst du eigene Fotos beisteuern.

## Mitmachen

NavigaTUM lebt davon, dass Leute Fehler melden und Lücken füllen.
Der einfachste Weg ist das Feedback-Formular auf jeder Seite - daraus entsteht ein öffentliches Issue [auf GitHub](https://github.com/TUM-Dev/NavigaTUM/issues), dessen Bearbeitung du nachverfolgen kannst.
Falsche Koordinaten oder Namen kannst du direkt auf der Detailseite bearbeiten und fehlende Einträge [vorschlagen](/propose).
Wer lieber an der Karte arbeitet, trägt fehlende Details direkt in [OpenStreetMap](https://www.openstreetmap.org) ein - sie erscheinen automatisch auch bei uns.
Und falls du mitentwickeln willst: [Der Code ist Open Source](https://github.com/TUM-Dev/NavigaTUM) und wir freuen uns über neue Mitwirkende.
