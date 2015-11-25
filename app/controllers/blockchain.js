"use strict";

var _                = require('underscore');
var Q                = require('q');
var co               = require('co');
var async            = require('async');
var es               = require('event-stream');
var moment           = require('moment');
var dos2unix         = require('../lib/dos2unix');
var http2raw         = require('../lib/streams/parsers/http2raw');
var jsoner           = require('../lib/streams/jsoner');
var http400          = require('../lib/http/http400');
var parsers          = require('../lib/streams/parsers/doc');
var blockchainDao    = require('../lib/blockchainDao');
var localValidator   = require('../lib/localValidator');
var globalValidator  = require('../lib/globalValidator');
var Membership       = require('../lib/entity/membership');

module.exports = function (server) {
  return new BlockchainBinding(server);
};

function BlockchainBinding (server) {

  var conf = server.conf;
  var local = localValidator(conf);
  var global = globalValidator(conf);

  // Services
  var ParametersService = server.ParametersService;
  var BlockchainService = server.BlockchainService;
  var IdentityService   = server.IdentityService;

  // Models
  var Block      = require('../lib/entity/block');
  var Stat       = require('../lib/entity/stat');

  this.parseMembership = function (req, res) {
    res.type('application/json');
    var onError = http400(res);
    http2raw.membership(req, onError)
      .pipe(dos2unix())
      .pipe(parsers.parseMembership(onError))
      .pipe(local.versionFilter(onError))
      .pipe(global.currencyFilter(onError))
      .pipe(server.singleWriteStream(onError))
      .pipe(jsoner())
      .pipe(es.stringify())
      .pipe(res);
  };

  this.parseBlock = function (req, res) {
    res.type('application/json');
    var onError = http400(res);
    http2raw.block(req, onError)
      .pipe(dos2unix())
      .pipe(parsers.parseBlock(onError))
      .pipe(local.versionFilter(onError))
      .pipe(global.currencyFilter(onError))
      .pipe(server.singleWriteStream(onError))
      .pipe(jsoner())
      .pipe(es.stringify())
      .pipe(res);
  };

  this.parameters = function (req, res) {
    res.type('application/json');
    server.dal.getParameters()
      .then(function(parameters){
        res.send(200, JSON.stringify(parameters, null, "  "));
      })
      .catch(function(){
        res.send(200, JSON.stringify({}, null, "  "));
      })
  };

  this.with = {

    newcomers: getStat('newcomers'),
    certs:     getStat('certs'),
    joiners:   getStat('joiners'),
    actives:   getStat('actives'),
    leavers:   getStat('leavers'),
    excluded:  getStat('excluded'),
    ud:        getStat('ud'),
    tx:        getStat('tx')
  };

  function getStat (statName) {
    return function (req, res) {
      async.waterfall([
        function (next) {
          server.dal.getStat(statName).then(_.partial(next, null)).catch(next);
        }
      ], function (err, stat) {
        if(err){
          res.send(400, err);
          return;
        }
        res.type('application/json');
        res.send(200, JSON.stringify({ result: new Stat(stat).json() }, null, "  "));
      });
    }
  }

  this.promoted = function (req, res) {
    res.type('application/json');
    async.waterfall([
      function (next){
        ParametersService.getNumber(req, next);
      },
      function (number, next){
        BlockchainService.promoted(number, next);
      }
    ], function (err, promoted) {
      if(err){
        res.send(404, err && (err.message || err));
        return;
      }
      res.send(200, JSON.stringify(new Block(promoted).json(), null, "  "));
    });
  };

  this.blocks = function (req, res) {
    res.type('application/json');
    co(function *() {
      try {
        let params = ParametersService.getCountAndFrom(req);
        var count = parseInt(params.count);
        var from = parseInt(params.from);
        let blocks = yield BlockchainService.blocksBetween(from, count);
        blocks = blocks.map((b) => (new Block(b).json()));
        res.send(200, JSON.stringify(blocks, null, "  "));
      } catch(e) {
        res.send(400, e);
      }
    });
  };

  this.current = function (req, res) {
    res.type('application/json');
    async.waterfall([
      function (next){
        BlockchainService.current(next);
      }
    ], function (err, current) {
      if(err || !current){
        res.send(404, err);
        return;
      }
      res.send(200, JSON.stringify(new Block(current).json(), null, "  "));
    });
  };

  this.hardship = function (req, res) {
    res.type('application/json');
    return co(function *() {
      let nextBlockNumber = 0;
      try {
        let search = yield ParametersService.getSearchP(req);
        let idty = yield IdentityService.findMemberWithoutMemberships(search);
        if (!idty) {
          throw 'Identity not found';
        }
        if (!idty.member) {
          throw 'Not a member';
        }
        let current = yield BlockchainService.current();
        if (current) {
          nextBlockNumber = current ? current.number + 1 : 0;
        }
        let nbZeros = yield globalValidator(conf, blockchainDao(null, server.dal)).getTrialLevel(idty.pubkey);
        res.send(200, JSON.stringify({
          "block": nextBlockNumber,
          "level": nbZeros
        }, null, "  "));
      } catch(e) {
        res.send(400, e);
      }
    });
  };

  this.memberships = function (req, res) {
    res.type('application/json');
    async.waterfall([
      function (next){
        ParametersService.getSearch(req, next);
      },
      function (search, next){
        IdentityService.findMember(search, next);
      }
    ], function (err, idty) {
      if(err){
        res.send(400, err);
        return;
      }
      var json = {
        pubkey: idty.pubkey,
        uid: idty.uid,
        sigDate: moment(idty.time).unix(),
        memberships: []
      };
      idty.memberships.forEach(function(ms){
        ms = new Membership(ms);
        json.memberships.push({
          version: ms.version,
          currency: conf.currency,
          membership: ms.membership,
          blockNumber: parseInt(ms.blockNumber),
          blockHash: ms.blockHash
        });
      });
      res.send(200, JSON.stringify(json, null, "  "));
    });
  };

  this.branches = function (req, res) {
    res.type('application/json');
    co(function *() {
      let branches = yield BlockchainService.branches();
      let blocks = branches.map((b) => new Block(b).json());
      res.send(200, JSON.stringify({
        blocks: blocks
      }, null, "  "));
    })
      .catch(function(err){
        console.error(err.stack || err.message || err);
        res.send(404, err && (err.message || err));
      });
  };
}
