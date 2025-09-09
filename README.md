# MV64e REST-to-Kafka Gateway

Diese Anwendung versendet MV64e HTTP requests mit DNPM V2.1 Payload an einen Kafka Broker

### Einordnung innerhalb einer DNPM-ETL-Strecke

Diese Anwendung erlaubt das Weiterleiten von REST Anfragen mit einem Request-Body und Inhalt im DNPM-Datenmodell 2.1
sowie `Content-Type` von `application/json` bzw `application/vnd.dnpm.v2.mtb+json` an einen Apache Kafka Cluster.

Verwendung im Zusammenspiel mit https://github.com/pcvolkmer/mv64e-etl-processor

![Modell DNPM-ETL-Strecke](docs/etl.png)

## Konfiguration

Beim Start der Anwendung können Parameter angegeben werden.

```
Usage: mv64e-rest-to-kafka-gateway [OPTIONS] --token <TOKEN>

Options:
      --listen <LISTEN>
          Address and port for HTTP requests [env: LISTEN=] [default: [::]:3000]
      --token <TOKEN>
          bcrypt hashed Security Token [env: SECURITY_TOKEN=]
      --bootstrap-server <BOOTSTRAP_SERVER>
          Kafka Bootstrap Server [env: KAFKA_BOOTSTRAP_SERVERS=] [default: kafka:9094]
      --topic <TOPIC>
          Kafka Topic [env: KAFKA_TOPIC=] [default: etl-processor_input]
      --ssl-ca-file <SSL_CA_FILE>
          CA file for SSL connection to Kafka [env: KAFKA_SSL_CA_FILE=]
      --ssl-cert-file <SSL_CERT_FILE>
          Certificate file for SSL connection to Kafka [env: KAFKA_SSL_CERT_FILE=]
      --ssl-key-file <SSL_KEY_FILE>
          Key file for SSL connection to Kafka [env: KAFKA_SSL_KEY_FILE=]
      --ssl-key-password <SSL_KEY_PASSWORD>
          The SSL key password [env: KAFKA_SSL_KEY_PASSWORD=]
```

Die Anwendung lässt sich auch mit Umgebungsvariablen konfigurieren.

* `LISTEN`: Adresse und Port für eingehende HTTP-Requests. Standardwert: `[::]:3000` - Port `3000` auf allen
  Adressen (IPv4 und IPv6)
* `SECURITY_TOKEN`: Verpflichtende Angabe des Benutzernamens und *bcrypt*-Hash des Passworts
* `KAFKA_BOOTSTRAP_SERVERS`: Zu verwendende Kafka-Bootstrap-Server als kommagetrennte Liste
* `KAFKA_TOPIC`: Zu verwendendes Topic zum Warten auf neue Anfragen. Standardwert: `etl-processor_input`

Optionale Umgebungsvariablen - wenn angegeben wird eine SSL-Verbindung zu Kafka aufgebaut.

* `KAFKA_SSL_CA_FILE`: CA für SSL-Verbindungen
* `KAFKA_SSL_CERT_FILE`: SSL Certificate Datei
* `KAFKA_SSL_KEY_FILE`: SSL Key Datei
* `KAFKA_SSL_KEY_PASSWORD`: SSL KEY Passwort (wenn benötigt)

Die Angabe eines Tokens ist verpflichtend und kann entweder über den Parameter `--token` erfolgen, oder über die
Umgebungsvariable `SECURITY_TOKEN`.

## HTTP-Requests

Die folgenden Endpunkte sind verfügbar:

* **POST** `/mtb/etl/patient-record`: Senden eines MTB-Files
* **DELETE** `/mtb/etl/patient-record/:patient_id`: Löschen von Informationen zu dem Patienten

Übermittelte MTB-Files müssen erforderliche Bestandteile beinhalten, ansonsten wird die Anfrage zurückgewiesen.

Zum Löschen von Patienteninformationen wird intern ein MTB-File mit Consent-Status `REJECTED` erzeugt und weiter
geleitet. Hier ist kein Request-Body erforderlich.

Bei Erfolg enthält die Antwort im HTTP-Header `x-request-id` die Anfrage-ID, die auch im ETL-Prozessor verwendet
wird.

### Authentifizierung

Requests müssen einen HTTP-Header `authorization` für HTTP-Basic enthalten.
Dazu muss die erforderliche Umgebungsvariable `SECURITY_TOKEN` gesetzt sein.
Der erforderliche Wert kann z.B. mit *htpasswd* erzeugt werden - `token` ist hier der gewählte Benutzername - es kann
jedoch ein beliebiger Benutzername verwendet werden:

```
htpasswd -Bn token
```

Der vordere Teil der Ausgabe ist der Benutzername, der hintere Teil (im Beispiel hinter `token:`) entspricht dem
*bcrypt*-Hash des Tokens, welches als HTTP-Basic-Passwort erwartet wird.

Ein Beispiel für die Angabe `SECURITY_TOKEN` ist für den Benutzernamen `token` und das Passwort `very-secret`:
`token:$2y$05$LIIFF4Rbi3iRVA4UIqxzPeTJ0NOn/cV2hDnSKFftAMzbEZRa42xSG`

Zur Kompatibilität mit älteren Versionen kann (nur) bei Wahl des Benutzernamens `token` der Teil `token:`
bei der Angabe entfallen: `$2y$05$LIIFF4Rbi3iRVA4UIqxzPeTJ0NOn/cV2hDnSKFftAMzbEZRa42xSG`

### Beispiele für HTTP-Requests und resultierende Kafka-Records

Beispiele für gültige HTTP-Requests zum Übermitteln und Löschen eines MTB-Files.

#### Übermittlung eines MTB-Files

Anfrage mit *curl*, hier mit beiliegendem Test-File:

```bash
curl -u token:very-secret \
  -H "Content-Type: application/json" \
  --data @test-files/mv64e-mtb-fake-patient.json \
  http://localhost:3000/mtb/etl/patient-record
```

Als Content-Type kann auch `application/vnd.dnpm.v2.mtb+json` verwendet werden.

Antwort:

```
HTTP/1.1 202 Accepted
x-request-id: 1804d5c1-af3d-4f75-81a0-d9ca7c9739ef
content-length: ...
date: Sat, 09 Mar 2024 11:16:44 GMT
```

Resultierender Kafka-Record:

* **Key**: `{ "pid" : "P1" }`
* **Headers**:
    * `requestId`: `1804d5c1-af3d-4f75-81a0-d9ca7c9739ef`
* **Value**: `{ "patient": { "id": "fae56ea7-24a7-4556-82fb-2b5dde71bb4d", .... } }`

#### Löschen von Patienten

Anfrage auch hier mit *curl*:

```bash
curl -u token:very-secret \
  -H "Content-Type: application/json" \
  -X DELETE \
  http://localhost:3000/mtb/etl/patient-record/P1
```

Antwort:

```
HTTP/1.1 202 Accepted
x-request-id: 8473fa67-8b18-4e8f-aa89-874f74fcc672
content-length: ...
date: Sat, 09 Mar 2024 11:24:35 GMT
```

Resultierender Kafka-Record:

* **Key**: `{ "pid" : "P1" }`
* **Headers**:
    * `requestId`: `8473fa67-8b18-4e8f-aa89-874f74fcc672`
* **Value**: JSON-String mit Patienten-ID `P1` und ohne weitere Angaben: `{ "patient": { "id": "P1", .... } }`

Es werden keine weiteren patientenbezogenen Daten übermittelt.

In optionaler Verbindung mit [Key-Based-Retention](https://github.com/pcvolkmer/mv64e-etl-processor#key-based-retention)
wird
lediglich der letzte und aktuelle Record, hier die Information ohne Consent-Zustimmung, in Kafka vorgehalten.

Trifft dieser Kafka-Record im [ETL-Prozessor](https://github.com/pcvolkmer/mv64e-etl-processor) ein, so wird dort
ebenfalls eine
Löschanfrage ausgelöst, da keine Modellvorhaben Metadaten enthalten sind.
