# DNPM Kafka Rest Proxy

DNPM Datenmodell 2.1 REST Proxy für Kafka

### Einordnung innerhalb einer DNPM-ETL-Strecke

Diese Anwendung erlaubt das Weiterleiten von REST Anfragen mit einem Request-Body und Inhalt im DNPM-Datenmodell 2.1
sowie `Content-Type` von `application/json` an einen Apache Kafka Cluster.

Verwendung im Zusammenspiel mit https://github.com/pcvolkmer/etl-processor

## Konfiguration

Beim Start der Anwendung können Parameter angegeben werden.

```
Usage: dnpm-kafka-rest-proxy [OPTIONS] --token <TOKEN>

Options:
      --bootstrap-server <BOOTSTRAP_SERVER>
          Kafka Bootstrap-Server(s) [env: KAFKA_BOOTSTRAP_SERVERS=] [default: kafka:9094]
      --topic <TOPIC>
          Kafka Topic [env: APP_KAFKA_TOPIC=] [default: etl-processor_input]
      --token <TOKEN>
          bcrypt hashed Security Token [env: APP_SECURITY_TOKEN=]
      --listen <LISTEN>
          Address and port for HTTP requests [env: APP_LISTEN=] [default: [::]:3000]
```

Die Anwendung lässt sich auch mit Umgebungsvariablen konfigurieren.

* `APP_KAFKA_SERVERS`: Zu verwendende Kafka-Bootstrap-Server als kommagetrennte Liste
* `APP_KAFKA_TOPIC`: Zu verwendendes Topic zum Warten auf neue Anfragen. Standardwert: `etl-processor_input`
* `APP_SECURITY_TOKEN`: Verpflichtende Angabe es Tokens als *bcrypt*-Hash
* `APP_LISTEN`: Adresse und Port für eingehende HTTP-Requests. Standardwert: `[::]:3000` - Port `3000` auf allen
  Adressen (IPv4 und IPv6)

Die Angabe eines Tokens ist verpflichtend und kann entweder über den Parameter `--token` erfolgen, oder über die
Umgebungsvariable `APP_SECURITY_TOKEN`.

## HTTP-Requests

Die folgenden Endpunkte sind verfügbar:

* **POST** `/mtbfile`: Senden eines MTB-Files
* **DELETE** `/mtbfile/:patient_id`: Löschen von Informationen zu dem Patienten

Übermittelte MTB-Files müssen erforderliche Bestandteile beinhalten, ansonsten wird die Anfrage zurückgewiesen.

Zum Löschen von Patienteninformationen wird intern ein MTB-File mit Consent-Status `REJECTED` erzeugt und weiter
geleitet. Hier ist kein Request-Body erforderlich.

Bei Erfolg enthält die Antwort im HTTP-Header `x-request-id` die Anfrage-ID, die auch im ETL-Prozessor verwendet
wird.

### Authentifizierung

Requests müssen einen HTTP-Header `authorization` für HTTP-Basic enthalten. Hier ist es erforderlich, dass der
Benutzername `token` gewählt wird.

Es ist hierzu erforderlich, die erforderliche Umgebungsvariable `APP_SECURITY_TOKEN` zu setzen. Dies kann z.B. mit
*htpasswd* erzeugt werden:

```
htpasswd -Bn token
```

Der hintere Teil (hinter `token:`) entspricht dem *bcrypt*-Hash des Tokens.

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

Antwort:

```
HTTP/1.1 202 Accepted
x-request-id: 1804d5c1-af3d-4f75-81a0-d9ca7c9739ef
content-length: ...
date: Sat, 09 Mar 2024 11:16:44 GMT
```

Resultierender Kafka-Record:

* **Key**: `{"pid":"P1"}`
* **Headers**:
    * `requestId`: `1804d5c1-af3d-4f75-81a0-d9ca7c9739ef`
* **Value**: Inhalt des HTTP-Request-Bodies/Test-Files

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

* **Key**: `{"pid":"P1"}`
* **Headers**:
    * `requestId`: `8473fa67-8b18-4e8f-aa89-874f74fcc672`
* **Value**: JSON-String mit Patienten-ID `P1` und ohne weitere Angaben.

Es werden keine weiteren patientenbezogenen Daten übermittelt.

In optionaler Verbindung mit [Key-Based-Retention](https://github.com/CCC-MF/etl-processor#key-based-retention) wird
lediglich der letzte und aktuelle Record, hier die Information ohne Consent-Zustimmung, in Kafka vorgehalten.

Trifft dieser Kafka-Record im [ETL-Prozessor](https://github.com/CCC-MF/etl-processor) ein, so wird dort ebenfalls eine
Löschanfrage ausgelöst, da keine Modellvorhaben Metadaten enthalten sind.
