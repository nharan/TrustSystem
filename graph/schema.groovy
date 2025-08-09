// JanusGraph schema for Trust & Reputation MVP
// Run in Gremlin Console connected to JanusGraph

mgmt = graph.openManagement()

// Property keys
did         = mgmt.makePropertyKey('did').dataType(String.class).cardinality(Cardinality.SINGLE).make()
handle      = mgmt.makePropertyKey('handle').dataType(String.class).make()
createdAt   = mgmt.makePropertyKey('createdAt').dataType(Long.class).make()
alpha       = mgmt.makePropertyKey('alpha').dataType(Long.class).make()
beta        = mgmt.makePropertyKey('beta').dataType(Long.class).make()
b           = mgmt.makePropertyKey('b').dataType(Float.class).make()
d           = mgmt.makePropertyKey('d').dataType(Float.class).make()
u           = mgmt.makePropertyKey('u').dataType(Float.class).make()
scope       = mgmt.makePropertyKey('scope').dataType(String.class).make()
domain      = mgmt.makePropertyKey('domain').dataType(String.class).make()
offenseScore= mgmt.makePropertyKey('offenseScore').dataType(Float.class).make()
botProb     = mgmt.makePropertyKey('botProb').dataType(Float.class).make()
strength    = mgmt.makePropertyKey('strength').dataType(Float.class).make()
weight      = mgmt.makePropertyKey('weight').dataType(Float.class).make()
ts          = mgmt.makePropertyKey('ts').dataType(Long.class).make()
evidenceRef = mgmt.makePropertyKey('evidenceRef').dataType(String.class).make()
cid         = mgmt.makePropertyKey('cid').dataType(String.class).cardinality(Cardinality.SINGLE).make()
authorDid   = mgmt.makePropertyKey('authorDid').dataType(String.class).make()
classification = mgmt.makePropertyKey('classification').dataType(String.class).make()
exp_map     = mgmt.makePropertyKey('exp_map').dataType(String.class).make() // optional JSON

// Vertex labels
user        = mgmt.makeVertexLabel('user').make()
content     = mgmt.makeVertexLabel('content').make()
community   = mgmt.makeVertexLabel('community').make()

// Edge labels
follows     = mgmt.makeEdgeLabel('follows').multiplicity(Multiplicity.MULTI).make()
trusts      = mgmt.makeEdgeLabel('trusts').multiplicity(Multiplicity.MULTI).make()
endorses    = mgmt.makeEdgeLabel('endorses').multiplicity(Multiplicity.MULTI).make()
interacts   = mgmt.makeEdgeLabel('interacts').multiplicity(Multiplicity.MULTI).make()

// Indexes
mgmt.buildIndex('userByDid', Vertex.class).addKey(did).unique().buildCompositeIndex()
mgmt.buildIndex('contentByAuthor', Vertex.class).addKey(authorDid).buildCompositeIndex()
mgmt.buildIndex('contentByCid', Vertex.class).addKey(cid).unique().buildCompositeIndex()
mgmt.buildIndex('userSearch', Vertex.class).addKey(handle).buildMixedIndex("search")
mgmt.buildIndex('contentSearch', Vertex.class).addKey(domain).addKey(classification).buildMixedIndex("search")

mgmt.commit()



