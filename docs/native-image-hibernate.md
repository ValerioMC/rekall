# Lessons learned — Hibernate su GraalVM native image (Spring Boot 4)

Stack: Spring Boot 4.1.0, Hibernate ORM 7.4.1.Final, Spring Framework 7.0.8, Java 25,
GraalVM 25.0.4, H2 su file, Liquibase, Maven multi-modulo.

## Causa radice

Hibernate usa ByteBuddy per generare bytecode **a runtime** (proxy di lazy-loading,
ottimizzatore di riflessione per l'istanziazione veloce delle entity). GraalVM native image
vieta sempre la generazione di classi a runtime, senza eccezioni configurabili. Serve quindi:

1. spegnere il bytecode provider di default (`bytebuddy`) a favore di `none`;
2. spostare a **build-time** ciò che ByteBuddy faceva a runtime (enhancement delle entity);
3. impedire che Hibernate costruisca comunque un'istanza di `BytecodeProviderImpl` (ByteBuddy)
   solo per enumerarla — il suo costruttore genera bytecode a runtime indipendentemente da
   quale provider verrà poi effettivamente usato.

A questi si aggiungono due gap di reflection metadata scoperti solo eseguendo davvero il
binario (non rilevabili a priori): uno su una classe di bootstrap dell'app, uno sui metodi
`$$_hibernate_*` che l'enhancement inietta nelle entity.

## Modifiche, file per file

### `rekall-app/src/main/resources/application.yaml`
Aggiunto `spring.jpa.properties.hibernate.bytecode.provider: none`.
**Perché**: senza questo, Hibernate chiede al provider di default (ByteBuddy) di costruire
un "reflection optimizer" per istanziare le entity — quel provider genera bytecode a runtime,
vietato da native image. Errore che produce se manca: `BeanCreationException` su
`entityManagerFactory`, causa `HibernateException: java.lang.InstantiationException:
<Entity>$HibernateInstantiator`.

### `rekall-domain/pom.xml`
Aggiunto profilo Maven `native` con `hibernate-maven-plugin` (goal `enhance`, versione pinnata
a `${hibernate.version}`, ereditata dal BOM di Spring Boot — mai lasciarla senza `<version>`:
senza pin la risoluzione Maven può prendere una versione del plugin diversa da quella di
hibernate-core, causando `AbstractMethodError` per bytecode incoerente).
**Perché**: con `bytecode.provider=none` non c'è più nessun generatore di proxy a runtime per
le associazioni `LAZY @ManyToOne`/`@OneToOne`. Il build-time enhancement inietta quella logica
direttamente nel `.class` compilato, così funziona anche con `none` a runtime. Gate dietro un
profilo (non sempre attivo) perché è overhead inutile per il normale run su JVM.

### `rekall-app/pom.xml`
Aggiunto profilo Maven `native` con:
- `spring-boot-maven-plugin` esecuzione `process-aot` (genera hint di reflection/proxy per
  Spring, inclusi quelli per repository JPA ed entity, dentro `target/classes`);
- `native-maven-plugin` con solo l'esecuzione `add-reachability-metadata` (scarica/allega la
  reachability metadata community per le dipendenze che la pubblicano).

**Deliberatamente NON contiene** l'esecuzione `compile-no-fork` che lancerebbe
`native-image` da dentro Maven: ogni tentativo di far arrivare a `native-maven-plugin` un
classpath con la copia patchata di `hibernate-core.jar` (vedi sotto) si è rivelato inaffidabile
con questa combinazione di plugin (shade-plugin legge l'artifact già trasformato in BOOT-INF/
da `spring-boot-maven-plugin`; `native-maven-plugin` non espande glob tipo `dir/*`; proprietà
Maven impostate da un plugin a metà build non sopravvivono fino alla config di
`native-maven-plugin`). La build vera è delegata a `scripts/native-build.sh`.

### `rekall-app/src/main/java/dev/rekall/RekallApplication.java`
Aggiunta `@RegisterReflectionForBinding` sulla classe `@SpringBootApplication`, con due gruppi
di classi:
- `DatabaseRegistry`, `DatabaseEntry` — deserializzate da un `ObjectMapper` semplice dentro
  `DatabaseRegistryStore`, chiamato da un `EnvironmentPostProcessor` che gira **prima** che il
  contesto Spring esista. Spring AOT non vede quel call site (non è un bean), quindi sotto
  native image serve registrare il binding a mano. Senza: `NoClassDefFoundError` /
  `InvalidDefinitionException: cannot deserialize ... this appears to be a native image`.
- `Company`, `Project`, `Task`, `TimeEntry`, `Wrapup`, `Document` (le entity JPA) — il
  build-time enhancement inietta nel bytecode metodi interni Hibernate (`$$_hibernate_*`:
  identity, dirty tracking, lazy loading). Gli hint AOT di Spring per JPA/Jackson coprono solo
  ciò che serve al binding (getter, setter, costruttori), non questi metodi interni a
  Hibernate: l'analisi a closed-world di GraalVM non li vede raggiungibili e li elimina,
  causando `AbstractMethodError` al primo utilizzo reale (es. un INSERT).

### `rekall-app/src/main/resources/META-INF/native-image/dev.rekall/rekall-app-agent-trace/reachability-metadata.json`
File generato con il tracing agent di GraalVM, eseguendo sulla **JVM normale** (jar già
pacchettizzato, non serve il profilo `native`/l'enhancement) un ciclo CRUD completo su tutti
gli endpoint REST, con un database vuoto così Liquibase applica tutte le migrazioni da zero
invece di trovarle già applicate.

Comando esatto usato:

```bash
mvn -pl rekall-app -am package -DskipTests   # jar JVM normale, niente -Pnative

mkdir -p /tmp/agent-trace /tmp/agent-home /tmp/agent-db

java \
  -Drekall.home=/tmp/agent-home \
  -agentlib:native-image-agent=config-output-dir=/tmp/agent-trace \
  -jar rekall-app/target/rekall-app-0.1.0-SNAPSHOT.jar \
  --server.port=18099
```

(`REKALL_DB_URL` puntato a un file H2 dentro `/tmp/agent-db`, così il DB parte vuoto — non è un
dettaglio specifico dell'agent, è la env var che questo progetto legge per la posizione del
database.)

Con il processo in esecuzione, dall'altro terminale: sequenza completa di richieste `curl`
contro `/api/companies`, `/api/projects`, `/api/tasks`, `/api/tasks/{id}/time-entries/start`
e `/stop`, `/api/tasks/{id}/wrapup`, `/api/documents`, `/api/export`, poi le `DELETE`
corrispondenti — cioè ogni operazione che l'app espone, non solo lo startup.

Poi **fermare il processo con un normale `kill` (SIGTERM), non `kill -9`**: l'agent scrive
`reachability-metadata.json` solo su shutdown pulito della JVM.

```bash
kill <pid>   # aspetta che il processo termini, poi:
mkdir -p rekall-app/src/main/resources/META-INF/native-image/dev.rekall/rekall-app-agent-trace
cp /tmp/agent-trace/reachability-metadata.json \
   rekall-app/src/main/resources/META-INF/native-image/dev.rekall/rekall-app-agent-trace/
```

La sottocartella `dev.rekall/rekall-app-agent-trace` è arbitraria (GraalVM scansiona
ricorsivamente qualunque `META-INF/native-image/**/*.json` sul classpath): il nome è scelto solo
per non entrare in conflitto con `dev.rekall/rekall-app`, la cartella che Spring Boot AOT
rigenera ad ogni build in `target/spring-aot/...` e poi copia in `target/classes`.

**Perché**: cattura in un colpo solo i bisogni di reflection che né Spring AOT né la
reachability metadata community coprono per questa combinazione di versioni — in particolare
`liquibase.change.core.AddUniqueConstraintChange.getDeferrable()`, usato da Liquibase per
calcolare il checksum dei changeset, chiamato riflessivamente per ogni sottoclasse `Change`
presente nei changelog. Rigenerare questo file (stesso comando, stesso giro di `curl`) se
cambiano changelog Liquibase, entity, o versione di Hibernate/Liquibase: il modo più affidabile
è rifare il trace, non indovinare le entry a mano.

### `scripts/native-build.sh` (nuovo)
Orchestra la build reale in 4 passi:
1. `mvn -Pnative -pl rekall-app -am clean install` — **`install`, non `package`**: i passi
   successivi girano come invocazioni Maven separate (non nella stessa sessione reactor) e
   risolvono `rekall-domain`/`rekall-api`/`rekall-mcp` da `~/.m2`. Con `package` la cache locale
   non viene aggiornata: si rischia di compilare la native image contro un jar vecchio (successo
   apparente a build-time, `AbstractMethodError` a runtime, e nessun modo di scoprirlo
   ispezionando `target/classes`, che è invece corretto).
2. `mvn dependency:build-classpath -DincludeScope=runtime` — costruisce il classpath completo
   escludendo le dipendenze di test (altrimenti `spring-boot-starter-test`/Mockito/JUnit finiscono
   nell'immagine nativa, causando `ClassNotFoundException` su classi di test non compilate per
   l'AOT dell'app).
3. Copia `hibernate-core-<versione>.jar` in `target/native-libs/` e rimuove da quella copia
   `META-INF/services/org.hibernate.bytecode.spi.BytecodeProvider` (`zip -d`) — **mai** dalla
   cache condivisa in `~/.m2`, perché l'`enhance` del passo 1 gira sulla JVM e ha bisogno di
   ByteBuddy funzionante per costruire l'enhancer.
4. Invoca `native-image` direttamente con quel classpath (copia patchata al posto
   dell'originale).

### `Makefile`
Aggiunto target `native: ui` → `./scripts/native-build.sh`. Coerente con gli altri target
(`build`, `jar`) che ricompilano prima il frontend.

## In una riga, per il prossimo progetto

Bytecode provider a `none` + build-time enhancement per le associazioni LAZY + una copia
locale di `hibernate-core.jar` senza la riga di ServiceLoader di ByteBuddy (mai sulla cache
condivisa) + `@RegisterReflectionForBinding` sulle entity enhanced + un giro di tracing agent
su un ciclo CRUD completo per Liquibase e affini + `mvn install`, non `package`, prima di
qualunque risoluzione del classpath fatta come comando Maven separato.
